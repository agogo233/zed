use gpui::actions;

// If the zed binary doesn't use anything in this crate, it will be optimized away
// and the actions won't initialize. So we just provide an empty initialization function
// to be called from main.
//
// These may provide relevant context:
// https://github.com/rust-lang/rust/issues/47384
// https://github.com/mmastrac/rust-ctor/issues/280
pub fn init() {}

actions!(
    menu,
    [
        /// 取消当前菜单操作
        Cancel,
        /// 确认选中的菜单项
        Confirm,
        /// 执行次要确认操作
        SecondaryConfirm,
        /// 选中菜单中的上一项
        SelectPrevious,
        /// 选中菜单中的下一项
        SelectNext,
        /// 选中菜单中的第一项
        SelectFirst,
        /// 选中菜单中的最后一项
        SelectLast,
        /// 进入子菜单（即导航至下级菜单）
        SelectChild,
        /// 退出子菜单（即导航至上级菜单）
        SelectParent,
        /// 从菜单开头重新开始
        Restart,
        EndSlot,
    ]
);
