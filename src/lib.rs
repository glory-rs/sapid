//! Composable, owner-scoped widgets built on Glory's semantic controls.
//!
//! Applications choose their Glory host independently. See the crate README for
//! controlled state, event ordering, theming, and current accessibility scope.
#![doc = include_str!("../README.md")]

mod accordion;
mod actions;
mod alert;
mod anchor;
mod app_shell;
mod appearance;
mod avatar;
mod button;
mod checkbox;
mod checkbox_group;
mod chip;
mod collapsible;
pub mod collection;
mod composite;
mod display;
mod document;
mod field;
mod form;
mod form_state;
mod grid_layout;
pub mod hint;
mod image;
mod indicator;
mod theme_icon;
mod input_group;
mod layout;
pub mod menu;
mod mixed_checkbox;
mod navigation;
pub mod native_select;
mod number_formatter;
mod overlay;
mod password_input;
mod radio;
mod scroll_area;
mod splitter;
mod segmented_control;
mod separator;
mod spoiler;
mod skeleton;
mod state;
mod stepper;
mod switch;
mod tabs;
mod text_field;
mod text_input;
mod textarea;
pub mod theme;
mod timeline;
mod toggle;
mod typography;
mod visually_hidden;

pub use accordion::{Accordion, AccordionItem, accordion, accordion_item};
pub use actions::{
    ActionIcon, Burger, ButtonGroup, CloseButton, action_icon, burger, button_group,
    close_button,
};
pub use alert::{Alert, AlertSeverity, Announcement, MAX_ALERT_TEXT_BYTES, alert};
pub use anchor::{Anchor, AnchorRequest, anchor};
pub use app_shell::*;
pub use avatar::{
    Avatar, AvatarGroup, AvatarShape, MAX_AVATAR_GROUP_ITEMS, avatar, avatar_group,
};
pub use button::{Button, ButtonVariant, button};
pub use checkbox::{Checkbox, checkbox};
pub use checkbox_group::{
    CheckboxGroup, CheckboxItem, MAX_CHECKBOX_GROUP_ITEMS, checkbox_group, checkbox_item,
};
pub use chip::{Chip, ChipGroup, ChipItem, ChipMode, chip, chip_group, chip_item};
pub use collapsible::{Collapsible, collapsible};
pub use collection::{
    Collection, CollectionLimits, CollectionMember, MountedItem, RovingFocus, Typeahead,
    TypeaheadConfig,
};
pub use composite::ContentPolicy;
pub use display::{
    Badge, BadgeVariant, Card, EmptyState, Paper, Pill, ShadowSize, badge, card, empty_state,
    paper, pill,
};
pub use document::{
    Blockquote, Code, DataList, DataListItem, Highlight, Item, Kbd, KbdKey, KbdPlatform,
    MAX_DOCUMENT_ITEMS, Mark, TextList, TextListItem, blockquote, code, data_list,
    data_list_item, highlight, item, kbd, mark, text_list, text_list_item,
};
pub use field::{
    Description, Field, FieldControl, FieldError, Fieldset, Label, description, field,
    field_description, field_error, fieldset, label,
};
pub use form::{Form, FormButton, form, reset_button, submit_button};
pub use form_state::*;
pub use grid_layout::{
    AspectRatio, Grid, GridItem, MAX_GRID_ITEMS, SimpleGrid, aspect_ratio, grid, grid_item,
    simple_grid,
};
pub use hint::{FloatingTarget, HoverCard, MAX_HINT_TEXT_BYTES, Tooltip, hover_card, tooltip};
pub use image::{BackgroundImage, Image, MAX_IMAGE_SOURCE_BYTES, background_image, image};
pub use indicator::{Indicator, IndicatorPosition, indicator};
pub use theme_icon::{Icon, ThemeIcon, ThemeIconVariant, theme_icon};
pub use input_group::{
    InputAddon, InputGroup, InputSection, input_addon, input_group, input_section,
};
pub use layout::{
    Box, Center, Container, Flex, Group, Space, Stack, box_, center, container, flex, group,
    space, stack,
};
pub use menu::{
    DropdownMenu, Menu, MenuEntry, MenuItem, MenuRadioGroup, MenuRadioItem, MenuSubmenu,
    dropdown_menu, menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
    menu_submenu,
};
pub use mixed_checkbox::{CheckState, MixedCheckbox, mixed_checkbox};
pub use navigation::*;
pub use native_select::{NativeSelect, NativeOption, NativeOptionGroup, native_select, select_option, select_group, MAX_NATIVE_SELECT_OPTIONS};
pub use number_formatter::{
    Currency, NumberFormat, NumberFormatter, NumberLocale, number_formatter,
};
pub use overlay::{
    AlertCancelReason, AlertDialog, Dialog, Drawer, DrawerSide, Popover, PopoverAlign,
    alert_dialog, dialog, drawer, popover,
};
pub use password_input::{PasswordInput, password_input};
pub use radio::{MAX_RADIO_GROUP_ITEMS, Radio, RadioGroup, radio, radio_group};
pub use scroll_area::*;
pub use splitter::*;
pub use segmented_control::{Segment, SegmentedControl, segment, segmented_control};
pub use separator::{Separator, separator};
pub use spoiler::{MAX_SPOILER_BYTES, Spoiler, spoiler};
pub use skeleton::{Skeleton, skeleton};
pub use stepper::*;
pub use switch::{Switch, switch};
pub use tabs::{Tab, TabActivation, Tabs, tab, tabs};
pub use text_field::{TextField, text_field};
pub use textarea::{Textarea, textarea};
pub use theme::{ColorScheme, Hue, Size, Theme};
pub use timeline::{
    MAX_TIMELINE_ITEMS, Timeline, TimelineItem, TimelineProgress, timeline, timeline_item,
};
pub use toggle::{
    MAX_TOGGLE_GROUP_ITEMS, Toggle, ToggleGroup, ToggleItem, ToggleMode, toggle, toggle_group,
    toggle_item,
};
pub use typography::{
    Text, TextVariant, Title, Typography, TypographyContext, text, title, typography,
};
pub use visually_hidden::{MAX_VISUALLY_HIDDEN_BYTES, VisuallyHidden, visually_hidden};

pub mod prelude {
    pub use crate::{
        Accordion, AccordionItem, ActionIcon, Anchor, AnchorRequest, AspectRatio, Badge,
        BadgeVariant, Blockquote, Box, BreadcrumbItem, Breadcrumbs, Burger, Button,
        ButtonGroup, ButtonVariant, Card, Center, CheckState, Checkbox, CheckboxGroup,
        CheckboxItem, Chip, ChipGroup, ChipItem, ChipMode, CloseButton, Code, Collapsible,
        Collection, CollectionLimits, CollectionMember, ColorScheme, Container, ContentPolicy,
        ControlValue, Currency, DataList, DataListItem, Description, Dialog, Drawer,
        DrawerSide, EmptyState, Field, FieldControl, FieldError, FieldOptions, Fieldset, Flex,
        Form, FormButton, FormEntry, FormError, FormField, FormFieldState, FormLimits,
        FormState, FormStatus, Grid, GridItem, Group, Highlight, Hue, InputAddon, InputGroup,
        InputSection, Item, Kbd, KbdKey, KbdPlatform, Label, Mark, MixedCheckbox, MountedItem,
        NavLink, NumberFormat, NumberFormatter, NumberLocale, PageItem, Pagination, Paper,
        PasswordInput, Pill, Popover, PopoverAlign, Radio, RadioGroup, RovingFocus, Segment,
        SegmentedControl, Separator, ShadowSize, SimpleGrid, Size, Space, Spoiler, Stack,
        SubmissionLimits, SubmitStatus, SubmitToken, Switch, Tab, TabActivation, Tabs, Text,
        TextField, TextList, TextListItem, TextVariant, Textarea, Theme, Timeline,
        TimelineItem, TimelineProgress, Title, Toggle, ToggleGroup, ToggleItem, ToggleMode,
        Typeahead, TypeaheadConfig, Typography, TypographyContext, ValidationErrors,
        ValidationFuture, accordion, accordion_item, action_icon, anchor, aspect_ratio, badge,
        blockquote, box_, breadcrumb_item, breadcrumbs, burger, button, button_group, card,
        center, checkbox, checkbox_group, checkbox_item, chip, chip_group, chip_item,
        close_button, code, collapsible, container, data_list, data_list_item, description,
        dialog, drawer, empty_state, field, field_description, field_error, fieldset, flex,
        form, grid, grid_item, group, highlight, input_addon, input_group, input_section, item,
        kbd, label, mark, mixed_checkbox, nav_link, number_formatter, pagination,
        pagination_window, paper, password_input, pill, popover, radio, radio_group,
        reset_button, segment, segmented_control, separator, simple_grid, space, spoiler,
        stack, submit_button, switch, tab, tabs, text, text_field, text_list, text_list_item,
        textarea, timeline, timeline_item, title, toggle, toggle_group, toggle_item,
        typography,
    };
    pub use crate::{
        AppShell, ShellContext, ShellLayout, ShellMode, Sidebar, SidebarContext, SidebarHandle,
        app_shell, sidebar,
    };
    pub use crate::{
        Avatar, AvatarGroup, AvatarShape, DropdownMenu, Menu, MenuEntry, MenuItem,
        MenuRadioGroup, MenuRadioItem, MenuSubmenu, VisuallyHidden, avatar, avatar_group,
        dropdown_menu, menu, menu_checkbox, menu_item, menu_radio_group, menu_radio_item,
        menu_submenu, visually_hidden,
    };
    pub use crate::{
        MAX_BREADCRUMB_ITEMS, MAX_PAGINATION_WINDOW_ITEMS, MAX_STEPPER_STEPS, Step, StepStatus,
        Stepper, step, stepper,
    };
    pub use crate::{
        MAX_PENDING_SCROLL_INTENTS, ScrollArea, ScrollContext, ScrollHandle, Scrollbars,
        Scroller, scroll_area, scroller,
    };
}
