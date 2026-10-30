use std::any::TypeId;

use debugger_panel::DebugPanel;
use editor::{Editor, MultiBufferOffsetUtf16};
use gpui::{Action, App, DispatchPhase, EntityInputHandler, TaskExt, actions};
use new_process_modal::{NewProcessModal, NewProcessMode};
use project::debugger::{self, breakpoint_store::SourceBreakpoint, session::ThreadStatus};
use schemars::JsonSchema;
use serde::Deserialize;
use session::DebugSession;

use tasks_ui::{Spawn, TaskOverrides};
use ui::{FluentBuilder, InteractiveElement};
use util::maybe;
use workspace::{ShutdownDebugAdapters, Workspace};
use zed_actions::debug_panel::{Toggle, ToggleFocus};

pub mod attach_modal;
pub mod debugger_panel;
mod dropdown_menus;
mod new_process_modal;
mod persistence;
pub(crate) mod session;

#[cfg(any(test, feature = "test-support"))]
pub mod tests;

// Let's see the diff-test in action.
actions!(
    debugger,
    [
        /// 启动新的调试会话
        Start,
        /// 继续执行所有线程，直到下一个断点
        Continue,
        /// 继续执行所选线程，直到下一个断点
        ContinueThread,
        /// 将调试器从正在运行的进程中分离
        Detach,
        /// 暂停当前运行的程序
        Pause,
        /// 重启当前调试会话
        Restart,
        /// 以相同配置重新运行当前调试会话
        RerunSession,
        /// 进入下一个函数调用
        StepInto,
        /// 单步跳过当前行
        StepOver,
        /// 跳出当前函数
        StepOut,
        /// 返回上一条语句
        StepBack,
        /// 停止调试会话
        Stop,
        /// 切换是否忽略所有断点
        ToggleIgnoreBreakpoints,
        /// 清除项目中的所有断点
        ClearAllBreakpoints,
        /// 聚焦到调试器控制台面板
        FocusConsole,
        /// 聚焦到变量面板
        FocusVariables,
        /// 聚焦到断点列表面板
        FocusBreakpointList,
        /// 聚焦到调用堆栈帧面板
        FocusFrames,
        /// 聚焦到已加载模块面板
        FocusModules,
        /// 聚焦到已加载源文件面板
        FocusLoadedSources,
        /// 聚焦到终端面板
        FocusTerminal,
        /// 切换线程选择器下拉菜单
        ToggleThreadPicker,
        /// 切换会话选择器下拉菜单
        ToggleSessionPicker,
        /// 重新运行上一次调试会话
        #[action(deprecated_aliases = ["debugger::RerunLastSession"])]
        Rerun,
        /// 切换调试器界面中所选项目的展开状态
        ToggleExpandItem,
        /// 切换堆栈帧列表中的用户帧筛选器
        /// 开启后，仅显示来自用户代码的帧
        /// 关闭后，显示所有帧
        ToggleUserFrames,
    ]
);

/// 在选定的变量或内存区域上设置数据断点
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = debugger)]
#[serde(deny_unknown_fields)]
pub struct ToggleDataBreakpoint {
    /// The type of data breakpoint
    /// Read & Write
    /// Read
    /// Write
    #[serde(default)]
    pub access_type: Option<dap::DataBreakpointAccessType>,
}

actions!(
    dev,
    [
        /// 将调试适配器启动参数复制到剪贴板
        CopyDebugAdapterArguments
    ]
);

pub fn init(cx: &mut App) {
    workspace::FollowableViewRegistry::register::<DebugSession>(cx);

    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace
            .register_action(spawn_task_or_modal)
            .register_action(|workspace, _: &ToggleFocus, window, cx| {
                workspace.toggle_panel_focus::<DebugPanel>(window, cx);
            })
            .register_action(|workspace, _: &Toggle, window, cx| {
                if !workspace.toggle_panel_focus::<DebugPanel>(window, cx) {
                    workspace.close_panel::<DebugPanel>(window, cx);
                }
            })
            .register_action(|workspace: &mut Workspace, _: &Start, window, cx| {
                NewProcessModal::show(workspace, window, NewProcessMode::Debug, None, cx);
            })
            .register_action(|workspace: &mut Workspace, _: &Rerun, window, cx| {
                let Some(debug_panel) = workspace.panel::<DebugPanel>(cx) else {
                    return;
                };

                debug_panel.update(cx, |debug_panel, cx| {
                    debug_panel.rerun_last_session(workspace, window, cx);
                })
            })
            .register_action(
                |workspace: &mut Workspace, _: &ShutdownDebugAdapters, _window, cx| {
                    workspace.project().update(cx, |project, cx| {
                        project.dap_store().update(cx, |store, cx| {
                            store.shutdown_sessions(cx).detach();
                        })
                    })
                },
            )
            .register_action_renderer(|div, workspace, _, cx| {
                let Some(debug_panel) = workspace.panel::<DebugPanel>(cx) else {
                    return div;
                };
                let Some(active_item) = debug_panel
                    .read(cx)
                    .active_session()
                    .map(|session| session.read(cx).running_state().clone())
                else {
                    return div;
                };
                let running_state = active_item.read(cx);
                if running_state.session().read(cx).is_terminated() {
                    return div;
                }

                let caps = running_state.capabilities(cx);
                let supports_step_back = caps.supports_step_back.unwrap_or_default();
                let supports_single_thread_execution_requests = caps
                    .supports_single_thread_execution_requests
                    .unwrap_or_default();
                let supports_detach = running_state.session().read(cx).is_attached();
                let status = running_state.thread_status(cx);

                let active_item = active_item.downgrade();
                div.when(status == Some(ThreadStatus::Running), |div| {
                    let active_item = active_item.clone();
                    div.on_action(move |_: &Pause, _, cx| {
                        active_item
                            .update(cx, |item, cx| item.pause_thread(cx))
                            .ok();
                    })
                })
                .when(status == Some(ThreadStatus::Stopped), |div| {
                    div.on_action({
                        let active_item = active_item.clone();
                        move |_: &StepInto, _, cx| {
                            active_item.update(cx, |item, cx| item.step_in(cx)).ok();
                        }
                    })
                    .on_action({
                        let active_item = active_item.clone();
                        move |_: &StepOver, _, cx| {
                            active_item.update(cx, |item, cx| item.step_over(cx)).ok();
                        }
                    })
                    .on_action({
                        let active_item = active_item.clone();
                        move |_: &StepOut, _, cx| {
                            active_item.update(cx, |item, cx| item.step_out(cx)).ok();
                        }
                    })
                    .when(supports_step_back, |div| {
                        let active_item = active_item.clone();
                        div.on_action(move |_: &StepBack, _, cx| {
                            active_item.update(cx, |item, cx| item.step_back(cx)).ok();
                        })
                    })
                    .on_action({
                        let active_item = active_item.clone();
                        move |_: &Continue, _, cx| {
                            active_item
                                .update(cx, |item, cx| item.continue_program(cx))
                                .ok();
                        }
                    })
                    .when(supports_single_thread_execution_requests, |div| {
                        let active_item = active_item.clone();
                        div.on_action(move |_: &ContinueThread, _, cx| {
                            active_item
                                .update(cx, |item, cx| item.continue_thread(cx))
                                .ok();
                        })
                    })
                })
                .when(supports_detach, |div| {
                    let active_item = active_item.clone();
                    div.on_action(move |_: &Detach, _, cx| {
                        active_item
                            .update(cx, |item, cx| item.detach_client(cx))
                            .ok();
                    })
                })
                .on_action({
                    let active_item = active_item.clone();
                    move |_: &Restart, _, cx| {
                        active_item
                            .update(cx, |item, cx| item.restart_session(cx))
                            .ok();
                    }
                })
                .on_action({
                    let active_item = active_item.clone();
                    move |_: &RerunSession, window, cx| {
                        active_item
                            .update(cx, |item, cx| item.rerun_session(window, cx))
                            .ok();
                    }
                })
                .on_action({
                    let active_item = active_item.clone();
                    move |_: &Stop, _, cx| {
                        active_item.update(cx, |item, cx| item.stop_thread(cx)).ok();
                    }
                })
                .on_action({
                    let active_item = active_item.clone();
                    move |_: &ToggleIgnoreBreakpoints, _, cx| {
                        active_item
                            .update(cx, |item, cx| item.toggle_ignore_breakpoints(cx))
                            .ok();
                    }
                })
                .on_action(move |_: &ToggleUserFrames, _, cx| {
                    if let Some((thread_status, stack_frame_list)) = active_item
                        .read_with(cx, |item, cx| {
                            (item.thread_status(cx), item.stack_frame_list().clone())
                        })
                        .ok()
                    {
                        stack_frame_list.update(cx, |stack_frame_list, cx| {
                            stack_frame_list.toggle_frame_filter(thread_status, cx);
                        })
                    }
                })
            });
    })
    .detach();

    cx.observe_new({
        move |editor: &mut Editor, _, _| {
            editor
                .register_action_renderer(move |editor, window, cx| {
                    let Some(workspace) = editor.workspace() else {
                        return;
                    };
                    let Some(debug_panel) = workspace.read(cx).panel::<DebugPanel>(cx) else {
                        return;
                    };
                    let Some(active_session) =
                        debug_panel.update(cx, |panel, _| panel.active_session())
                    else {
                        return;
                    };

                    let session = active_session
                        .read(cx)
                        .running_state
                        .read(cx)
                        .session()
                        .read(cx);

                    if session.is_terminated() {
                        return;
                    }

                    let editor = cx.entity().downgrade();

                    window.on_action_when(
                        session.any_stopped_thread(),
                        TypeId::of::<editor::actions::RunToCursor>(),
                        {
                            let editor = editor.clone();
                            let active_session = active_session.clone();
                            move |_, phase, _, cx| {
                                if phase != DispatchPhase::Bubble {
                                    return;
                                }
                                maybe!({
                                    let (buffer, position) = editor
                                        .update(cx, |editor, cx| {
                                            let cursor_point: language::Point = editor
                                                .selections
                                                .newest(&editor.display_snapshot(cx))
                                                .head();

                                            editor
                                                .buffer()
                                                .read(cx)
                                                .point_to_buffer_point(cursor_point, cx)
                                        })
                                        .ok()??;

                                    let path =
                                debugger::breakpoint_store::BreakpointStore::abs_path_from_buffer(
                                    &buffer, cx,
                                )?;

                                    let source_breakpoint = SourceBreakpoint {
                                        row: position.row,
                                        path,
                                        message: None,
                                        condition: None,
                                        hit_condition: None,
                                        state: debugger::breakpoint_store::BreakpointState::Enabled,
                                    };

                                    active_session.update(cx, |session, cx| {
                                        session.running_state().update(cx, |state, cx| {
                                            if let Some(thread_id) = state.selected_thread_id() {
                                                state.session().update(cx, |session, cx| {
                                                    session.run_to_position(
                                                        source_breakpoint,
                                                        thread_id,
                                                        cx,
                                                    );
                                                })
                                            }
                                        });
                                    });

                                    Some(())
                                });
                            }
                        },
                    );

                    window.on_action(
                        TypeId::of::<editor::actions::EvaluateSelectedText>(),
                        move |_, _, window, cx| {
                            let status = maybe!({
                                let text = editor
                                    .update(cx, |editor, cx| {
                                        let range = editor
                                            .selections
                                            .newest::<MultiBufferOffsetUtf16>(
                                                &editor.display_snapshot(cx),
                                            )
                                            .range();
                                        editor.text_for_range(
                                            range.start.0.0..range.end.0.0,
                                            &mut None,
                                            window,
                                            cx,
                                        )
                                    })
                                    .ok()??;

                                active_session.update(cx, |session, cx| {
                                    session.running_state().update(cx, |state, cx| {
                                        let stack_id = state.selected_stack_frame_id(cx);

                                        state.session().update(cx, |session, cx| {
                                            session
                                                .evaluate(
                                                    text,
                                                    Some(dap::EvaluateArgumentsContext::Repl),
                                                    stack_id,
                                                    None,
                                                    cx,
                                                )
                                                .detach();
                                        });
                                    });
                                });

                                Some(())
                            });
                            if status.is_some() {
                                cx.stop_propagation();
                            }
                        },
                    );
                })
                .detach();
        }
    })
    .detach();
}

fn spawn_task_or_modal(
    workspace: &mut Workspace,
    action: &Spawn,
    window: &mut ui::Window,
    cx: &mut ui::Context<Workspace>,
) {
    match action {
        Spawn::ByName {
            task_name,
            reveal_target,
        } => {
            let overrides = reveal_target.map(|reveal_target| TaskOverrides {
                reveal_target: Some(reveal_target),
            });
            let name = task_name.clone();
            tasks_ui::spawn_tasks_filtered(
                move |(_, task)| task.label.eq(&name),
                overrides,
                window,
                cx,
            )
            .detach_and_log_err(cx)
        }
        Spawn::ByTag {
            task_tag,
            reveal_target,
        } => {
            let overrides = reveal_target.map(|reveal_target| TaskOverrides {
                reveal_target: Some(reveal_target),
            });
            let tag = task_tag.clone();
            tasks_ui::spawn_tasks_filtered(
                move |(_, task)| task.tags.contains(&tag),
                overrides,
                window,
                cx,
            )
            .detach_and_log_err(cx)
        }
        Spawn::ViaModal { reveal_target } => {
            NewProcessModal::show(workspace, window, NewProcessMode::Task, *reveal_target, cx);
        }
    }
}
