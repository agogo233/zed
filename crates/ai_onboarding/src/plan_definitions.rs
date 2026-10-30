use gpui::{IntoElement, ParentElement};
use ui::{List, ListBulletItem, prelude::*};

/// Centralized definitions for Zed AI plans
pub struct PlanDefinitions;

impl PlanDefinitions {
    pub fn free_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(
                "使用你的 AI API 密钥，无限次发送提示词",
            ))
            .child(ListBulletItem::new("无限制使用外部智能体"))
    }

    pub fn sign_in_upsell(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制编辑预测"))
            .child(ListBulletItem::new("GPT Luna $5 额度"))
            .child(ListBulletItem::new("无需信用卡"))
    }

    pub fn pro_trial(&self, period: bool) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("GPT Luna $5 额度"))
            .child(ListBulletItem::new("无限制编辑预测"))
            .when(period, |this| {
                this.child(ListBulletItem::new(
                    "自试用开始起 14 天，无需信用卡",
                ))
            })
    }

    pub fn pro_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("Zed 智能体 $5 额度"))
            .child(ListBulletItem::new("超出 $5 后按用量计费"))
            .child(ListBulletItem::new("无限制编辑预测"))
    }

    pub fn business_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制编辑预测"))
            .child(ListBulletItem::new("按用量计费"))
    }

    pub fn vip_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制编辑预测"))
            .child(ListBulletItem::new("Zed 智能体中的 Token"))
    }

    pub fn student_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制编辑预测"))
            .child(ListBulletItem::new("Zed 智能体 $10 额度"))
            .child(ListBulletItem::new(
                "可选的额外用量点数包",
            ))
    }
}
