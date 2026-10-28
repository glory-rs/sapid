//! Explicit mount-time environment. Factories receive values directly; there
//! is no ambient global registry or automatic mutation of existing controls.
use crate::{Size, Theme, TypographyContext, TypographyRole};
use glory::render::{Direction, Environment, LanguageTag, MotionPreference, NodeId, Semantics};
use glory::style::{Family, Style};
use glory::ui::{Children, IntoChildren, Result, Widget};
use glory::{Cage, Scope, Vary};
use std::sync::{Arc, OnceLock};

pub const MAX_LOCALE_TEXT_BYTES: usize = 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocaleMessage {
    Open,
    Close,
    Previous,
    Next,
    Clear,
    Search,
    NoResults,
    Remove,
    Cancel,
    Confirm,
    Loading,
    Selected,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleMessages {
    pub open: String,
    pub close: String,
    pub previous: String,
    pub next: String,
    pub clear: String,
    pub search: String,
    pub no_results: String,
    pub remove: String,
    pub cancel: String,
    pub confirm: String,
    pub loading: String,
    pub selected: String,
}
impl LocaleMessages {
    fn get(&self, key: LocaleMessage) -> &str {
        match key {
            LocaleMessage::Open => &self.open,
            LocaleMessage::Close => &self.close,
            LocaleMessage::Previous => &self.previous,
            LocaleMessage::Next => &self.next,
            LocaleMessage::Clear => &self.clear,
            LocaleMessage::Search => &self.search,
            LocaleMessage::NoResults => &self.no_results,
            LocaleMessage::Remove => &self.remove,
            LocaleMessage::Cancel => &self.cancel,
            LocaleMessage::Confirm => &self.confirm,
            LocaleMessage::Loading => &self.loading,
            LocaleMessage::Selected => &self.selected,
        }
    }
    fn validate(&self) -> Result {
        for text in [
            &self.open,
            &self.close,
            &self.previous,
            &self.next,
            &self.clear,
            &self.search,
            &self.no_results,
            &self.remove,
            &self.cancel,
            &self.confirm,
            &self.loading,
            &self.selected,
        ] {
            if text.len() > MAX_LOCALE_TEXT_BYTES
                || text.trim().is_empty()
                || text.contains(['\0', '\r', '\n'])
            {
                return Err(glory::render::Error::InvalidProps("invalid locale message").into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locale {
    language: LanguageTag,
    messages: Arc<LocaleMessages>,
}
impl Default for Locale {
    fn default() -> Self {
        Self::english()
    }
}
impl Locale {
    pub fn new(language: LanguageTag, messages: LocaleMessages) -> Result<Self> {
        messages.validate()?;
        Ok(Self {
            language,
            messages: Arc::new(messages),
        })
    }
    pub fn language(&self) -> &LanguageTag {
        &self.language
    }
    pub fn message(&self, key: LocaleMessage) -> &str {
        self.messages.get(key)
    }
    pub fn english() -> Self {
        static LOCALE: OnceLock<Locale> = OnceLock::new();
        LOCALE
            .get_or_init(|| {
                Self::new(
                    LanguageTag::new("en-US").unwrap(),
                    LocaleMessages {
                        open: "Open".into(),
                        close: "Close".into(),
                        previous: "Previous".into(),
                        next: "Next".into(),
                        clear: "Clear".into(),
                        search: "Search".into(),
                        no_results: "No results".into(),
                        remove: "Remove".into(),
                        cancel: "Cancel".into(),
                        confirm: "Confirm".into(),
                        loading: "Loading".into(),
                        selected: "Selected".into(),
                    },
                )
                .unwrap()
            })
            .clone()
    }
    pub fn chinese() -> Self {
        static LOCALE: OnceLock<Locale> = OnceLock::new();
        LOCALE
            .get_or_init(|| {
                Self::new(
                    LanguageTag::new("zh-CN").unwrap(),
                    LocaleMessages {
                        open: "打开".into(),
                        close: "关闭".into(),
                        previous: "上一个".into(),
                        next: "下一个".into(),
                        clear: "清空".into(),
                        search: "搜索".into(),
                        no_results: "没有结果".into(),
                        remove: "移除".into(),
                        cancel: "取消".into(),
                        confirm: "确认".into(),
                        loading: "加载中".into(),
                        selected: "已选择".into(),
                    },
                )
                .unwrap()
            })
            .clone()
    }
    pub fn arabic() -> Self {
        static LOCALE: OnceLock<Locale> = OnceLock::new();
        LOCALE
            .get_or_init(|| {
                Self::new(
                    LanguageTag::new("ar").unwrap(),
                    LocaleMessages {
                        open: "فتح".into(),
                        close: "إغلاق".into(),
                        previous: "السابق".into(),
                        next: "التالي".into(),
                        clear: "مسح".into(),
                        search: "بحث".into(),
                        no_results: "لا توجد نتائج".into(),
                        remove: "إزالة".into(),
                        cancel: "إلغاء".into(),
                        confirm: "تأكيد".into(),
                        loading: "جار التحميل".into(),
                        selected: "محدد".into(),
                    },
                )
                .unwrap()
            })
            .clone()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct UiEnvironmentContext {
    pub theme: Theme,
    pub direction: Direction,
    pub locale: Locale,
    /// Reduce explicitly disables requested motion. NoPreference still permits
    /// the host's OS preference to reduce an animation.
    pub motion: MotionPreference,
}
impl Default for UiEnvironmentContext {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            direction: Direction::Ltr,
            locale: Locale::default(),
            motion: MotionPreference::NoPreference,
        }
    }
}
impl UiEnvironmentContext {
    pub fn message(&self, key: LocaleMessage) -> &str {
        self.locale.message(key)
    }
    pub fn spacing(&self, size: Size) -> f64 {
        self.theme.spacing(size)
    }
    pub fn typography(&self, size: Size) -> TypographyContext {
        TypographyContext {
            theme: self.theme.clone(),
            size,
            unstyled: false,
        }
    }
    pub fn transition_duration(&self, milliseconds: u32) -> Result<u32> {
        if milliseconds > 2000 {
            return Err(glory::render::Error::InvalidProps(
                "transition duration must be at most 2000ms",
            )
            .into());
        }
        Ok(if self.motion == MotionPreference::Reduce {
            0
        } else {
            milliseconds
        })
    }
    pub fn provider(&self) -> ThemeProvider {
        ThemeProvider::new(self.clone(), true)
    }
}
type Content = Box<dyn FnMut(UiEnvironmentContext) -> Children>;
pub struct ThemeProvider {
    context: UiEnvironmentContext,
    theme_style: bool,
    class: Vary<String>,
    style: Vary<Style>,
    node_ref: Option<Cage<Option<NodeId>>>,
    content: Option<Content>,
    children: Option<Children>,
}
pub fn theme_provider(theme: Theme) -> ThemeProvider {
    ThemeProvider::new(
        UiEnvironmentContext {
            theme,
            ..Default::default()
        },
        true,
    )
}
pub fn direction_provider(direction: Direction) -> ThemeProvider {
    ThemeProvider::new(
        UiEnvironmentContext {
            direction,
            ..Default::default()
        },
        false,
    )
}
pub fn locale_provider(locale: Locale) -> ThemeProvider {
    ThemeProvider::new(
        UiEnvironmentContext {
            locale,
            ..Default::default()
        },
        false,
    )
}
impl ThemeProvider {
    fn new(context: UiEnvironmentContext, theme_style: bool) -> Self {
        Self {
            context,
            theme_style,
            class: "".into(),
            style: Style::new().into(),
            node_ref: None,
            content: None,
            children: None,
        }
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.context.direction = direction;
        self
    }
    pub fn locale(mut self, locale: Locale) -> Self {
        self.context.locale = locale;
        self
    }
    pub fn motion(mut self, motion: MotionPreference) -> Self {
        self.context.motion = motion;
        self
    }
    pub fn class(mut self, class: impl Into<Vary<String>>) -> Self {
        self.class = class.into();
        self
    }
    pub fn style(mut self, style: impl Into<Vary<Style>>) -> Self {
        self.style = style.into();
        self
    }
    pub fn node_ref(mut self, node_ref: Cage<Option<NodeId>>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }
    pub fn fill(mut self, children: impl IntoChildren) -> Self {
        self.children = Some(children.into_children());
        self
    }
    pub fn content<C: IntoChildren>(
        mut self,
        mut content: impl FnMut(UiEnvironmentContext) -> C + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |context| content(context).into_children()));
        self
    }
}
impl Widget for ThemeProvider {
    fn build(&mut self, scope: &Scope) -> Result {
        self.context.theme.validate()?;
        if self.content.is_some() && self.children.is_some() {
            return Err(glory::render::Error::InvalidProps(
                "provider accepts fill or content, not both",
            )
            .into());
        }
        let mut base = Style::new();
        if self.theme_style {
            let metrics = self
                .context
                .theme
                .typography(TypographyRole::Body, Size::Medium);
            base = Style {
                background: Some(self.context.theme.canvas),
                color: Some(self.context.theme.foreground),
                font_size: Some(metrics.font_size),
                line_height: Some(metrics.line_height),
                font_weight: metrics.weight,
                ..Style::new().font_family([Family::System])
            };
        }
        let custom = crate::composite::validated(scope, &self.style, |style| {
            style
                .validate()
                .map_err(glory::render::Error::InvalidProps)?;
            Ok(())
        })?;
        let style = scope.bond(move || crate::appearance::merge(base.clone(), custom.get()));
        let extra = self.class.clone();
        let prefix = self.context.theme.class();
        let class =
            scope.bond(move || format!("sapid-theme-provider {prefix} {}", extra.get()));
        let mut root = glory::ui::view()
            .class(class)
            .style(style)
            .semantics(Some(Box::new(Semantics {
                environment: Some(Environment {
                    direction: self.context.direction,
                    language: self.context.locale.language().clone(),
                }),
                ..Default::default()
            })));
        if let Some(node_ref) = self.node_ref {
            root = root.node_ref(node_ref);
        }
        if let Some(mut content) = self.content.take() {
            root = root.fill(content(self.context.clone()));
        }
        if let Some(children) = self.children.take() {
            root = root.fill(children);
        }
        root.show_in(scope)
    }
}
