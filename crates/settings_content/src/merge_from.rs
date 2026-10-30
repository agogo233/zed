/// 递归合并设置结构的 trait
///
/// Zed 启动时从 `default.json` 加载设置以初始化
/// 所有设置。这些设置可通过加载用户设置进一步调整，
/// 以及任何设置配置文件；然后再通过加载任何
/// 本地项目设置。
///
/// 合并的默认行为如下：
/// * 对于具有命名键的对象（HashMap、结构体等），值将被深度合并
///   （例如，若默认设置中 languages.JSON.prettier.allowed = true，而用户设置中
///    languages.JSON.tab_size = 4；合并后的设置文件将同时包含两个设置）。
/// * 对于 Option 类型，None 值将被忽略，Some 值将被递归合并。
/// * 对于其他类型（包括 Vec），合并将覆盖当前值。
///
/// 如果需要打破规则，可以使用相关类型（如 ExtendingVec 或 SaturatingBool）。
#[allow(unused)]
pub trait MergeFrom {
    /// 从相同类型的源合并
    fn merge_from(&mut self, other: &Self);

    /// 从相同类型的可选源合并
    fn merge_from_option(&mut self, other: Option<&Self>) {
        if let Some(other) = other {
            self.merge_from(other);
        }
    }
}

macro_rules! merge_from_overwrites {
    ($($type:ty),+ $(,)?) => {
        $(
            impl MergeFrom for $type {
                fn merge_from(&mut self, other: &Self) {
                    *self = other.clone();
                }
            }
        )+
    }
}

merge_from_overwrites!(
    u16,
    u32,
    u64,
    usize,
    i16,
    i32,
    i64,
    bool,
    f64,
    f32,
    char,
    std::num::NonZeroUsize,
    std::num::NonZeroU32,
    String,
    std::sync::Arc<str>,
    std::path::PathBuf,
    std::sync::Arc<std::path::Path>,
    language_model_core::Speed,
);

impl<T: Clone + MergeFrom> MergeFrom for Option<T> {
    fn merge_from(&mut self, other: &Self) {
        let Some(other) = other else {
            return;
        };

        if let Some(this) = self {
            this.merge_from(other);
        } else {
            self.replace(other.clone());
        }
    }
}

impl<T: Clone> MergeFrom for Vec<T> {
    fn merge_from(&mut self, other: &Self) {
        *self = other.clone()
    }
}

impl<T: MergeFrom> MergeFrom for Box<T> {
    fn merge_from(&mut self, other: &Self) {
        self.as_mut().merge_from(other.as_ref())
    }
}

// Implementations for collections that extend/merge their contents
impl<K, V> MergeFrom for collections::HashMap<K, V>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone + MergeFrom,
{
    fn merge_from(&mut self, other: &Self) {
        for (key, value) in other {
            if let Some(existing) = self.get_mut(key) {
                existing.merge_from(value);
            } else {
                self.insert(key.clone(), value.clone());
            }
        }
    }
}

impl<K, V> MergeFrom for collections::BTreeMap<K, V>
where
    K: Clone + std::hash::Hash + Eq + Ord,
    V: Clone + MergeFrom,
{
    fn merge_from(&mut self, other: &Self) {
        for (key, value) in other {
            if let Some(existing) = self.get_mut(key) {
                existing.merge_from(value);
            } else {
                self.insert(key.clone(), value.clone());
            }
        }
    }
}

impl<K, V> MergeFrom for collections::IndexMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone + MergeFrom,
{
    fn merge_from(&mut self, other: &Self) {
        for (key, value) in other {
            if let Some(existing) = self.get_mut(key) {
                existing.merge_from(value);
            } else {
                self.insert(key.clone(), value.clone());
            }
        }
    }
}

impl<T> MergeFrom for collections::BTreeSet<T>
where
    T: Clone + Ord,
{
    fn merge_from(&mut self, other: &Self) {
        for item in other {
            self.insert(item.clone());
        }
    }
}

impl<T> MergeFrom for collections::HashSet<T>
where
    T: Clone + std::hash::Hash + Eq,
{
    fn merge_from(&mut self, other: &Self) {
        for item in other {
            self.insert(item.clone());
        }
    }
}

impl MergeFrom for serde_json::Value {
    fn merge_from(&mut self, other: &Self) {
        match (self, other) {
            (serde_json::Value::Object(this), serde_json::Value::Object(other)) => {
                for (key, value) in other {
                    if let Some(existing) = this.get_mut(key) {
                        existing.merge_from(value);
                    } else {
                        this.insert(key.clone(), value.clone());
                    }
                }
            }
            (this, other) => *this = other.clone(),
        }
    }
}
