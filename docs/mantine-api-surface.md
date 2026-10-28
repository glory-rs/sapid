# Mantine API 表面与本地入口索引

[决策与差异报告](mantine-api-comparison.md)。

## 完整 API 表面索引（候选不是等价性结论）

每项记录官方属性、功能章节、Sapid 对应实现路径和公开入口。共享文件可能包含同族其他 builder；
下列入口列表是导航证据，不是某个 builder 支持所有入口的声明。受控回调、Field 组合、Style 声明和宿主限制须结合上表阅读。

### AppShell — M:AppShell

来源：[@mantine/core](https://mantine.dev/core/app-shell/)；对应 `P6-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `aside`, `disabled`, `footer`, `grow`, `header`, `layout`, `mode`, `navbar`, `offsetScrollbars`, `padding`, `resize`, `transitionDuration`, `transitionTimingFunction`, `withBorder`, `zIndex`

**官方功能章节：** Examples；Usage；AppShell components；Configuration；layout prop；Resizable sections；Height configuration；Width configuration；padding prop；Header offset configuration；Collapsed navbar/aside configuration；withBorder prop；zIndex prop；Control transitions；disabled prop；AppShell.Section component；Semantic elements；CSS variables

**本地实现：** [src/app_shell.rs](../src/app_shell.rs)

**本地公开入口（同族）：** `app_shell`, `breakpoint`, `class`, `collapsed`, `content`, `default_collapsed`, `default_mobile_open`, `drawer_ref`, `footer`, `header`, `initial_width`, `layout`, `main`, `main_ref`, `mobile`, `mobile_open`, `name`, `nav_ref`, `new`, `node_ref`, `observe_width`, `on`, `on_collapsed_change`, `on_mobile_open_change`, `on_width_change`, `open`, `rail_width`, `set_collapsed`, `set_open`, `sidebar`, `size`, `skip_label`, `style`, `theme`, `toggle`, `unstyled`, `width`

**需要语义映射的属性候选：** `aside`, `disabled`, `grow`, `mode`, `navbar`, `offsetScrollbars`, `padding`, `resize`, `transitionDuration`, `transitionTimingFunction`, `withBorder`, `zIndex`

### AspectRatio — M:AspectRatio

来源：[@mantine/core](https://mantine.dev/core/aspect-ratio/)；对应 `P3-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `ratio`

**官方功能章节：** Usage；Map embed；Video embed；Inside flex container

**本地实现：** [../glory/crates/style/src/grid.rs](../../glory/crates/style/src/grid.rs)、[src/grid_layout.rs](../src/grid_layout.rs)

**本地公开入口（同族）：** `aspect_ratio`, `class`, `column`, `column_tracks`, `columns`, `fill`, `gap`, `grid`, `grid_item`, `height`, `item`, `items`, `name`, `node_ref`, `row`, `row_span`, `row_tracks`, `simple_grid`, `size`, `span`, `style`, `theme`, `unstyled`, `width`

**需要语义映射的属性候选：** `ratio`

### Center — M:Center

来源：[@mantine/core](https://mantine.dev/core/center/)；对应 `P3-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `inline`

**官方功能章节：** Usage；Inline

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)、[src/responsive.rs](../src/responsive.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `breakpoint`, `breakpoints`, `class`, `container`, `content`, `context_ref`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pending_queries`, `pointer_events`, `position`, `radius`, `resolve`, `responsive_root`, `right`, `scroll_anchor`, `select`, `size`, `style`, `styles`, `text_overflow`, `theme`, `thresholds`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** `children`, `inline`

### Container — M:Container

来源：[@mantine/core](https://mantine.dev/core/container/)；对应 `P3-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `fluid`, `size`, `strategy`

**官方功能章节：** Grid strategy；Usage；Fluid；Customize sizes；Responsive max-width

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)、[src/responsive.rs](../src/responsive.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `breakpoint`, `breakpoints`, `class`, `container`, `content`, `context_ref`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pending_queries`, `pointer_events`, `position`, `radius`, `resolve`, `responsive_root`, `right`, `scroll_anchor`, `select`, `size`, `style`, `styles`, `text_overflow`, `theme`, `thresholds`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** `strategy`

### Flex — M:Flex

来源：[@mantine/core](https://mantine.dev/core/flex/)；对应 `P3-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `align`, `columnGap`, `direction`, `gap`, `justify`, `rowGap`, `wrap`

**官方功能章节：** Usage；Supported props；Responsive props；Difference from Group and Stack；Browser support

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `class`, `container`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pointer_events`, `position`, `radius`, `right`, `scroll_anchor`, `size`, `style`, `text_overflow`, `theme`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** `columnGap`, `rowGap`

### Grid — M:Grid

来源：[@mantine/core](https://mantine.dev/core/grid/)；对应 `P3-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `align`, `breakpoints`, `columnGap`, `columns`, `gap`, `grow`, `justify`, `offset`, `order`, `overflow`, `rowGap`, `span`, `type`

**官方功能章节：** Usage；Columns span；Gap；Row and column gap；Grow；Column offset；Order；Multiple rows；Justify and align；Column alignment；Auto sized columns；Fit column content；Change columns count；Container queries；Browser support

**本地实现：** [../glory/crates/style/src/grid.rs](../../glory/crates/style/src/grid.rs)、[src/grid_layout.rs](../src/grid_layout.rs)、[src/responsive.rs](../src/responsive.rs)

**本地公开入口（同族）：** `aspect_ratio`, `breakpoint`, `breakpoints`, `class`, `column`, `column_tracks`, `columns`, `content`, `context_ref`, `fill`, `gap`, `grid`, `grid_item`, `height`, `item`, `items`, `name`, `new`, `node_ref`, `pending_queries`, `resolve`, `responsive_root`, `row`, `row_span`, `row_tracks`, `select`, `simple_grid`, `size`, `span`, `style`, `styles`, `theme`, `thresholds`, `unstyled`, `width`

**需要语义映射的属性候选：** `align`, `columnGap`, `grow`, `justify`, `offset`, `order`, `overflow`, `rowGap`, `type`

### Group — M:Group

来源：[@mantine/core](https://mantine.dev/core/group/)；对应 `P3-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `align`, `gap`, `grow`, `justify`, `preventGrowOverflow`, `wrap`

**官方功能章节：** Usage；preventGrowOverflow；Group children；Browser support

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `class`, `container`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pointer_events`, `position`, `radius`, `right`, `scroll_anchor`, `size`, `style`, `text_overflow`, `theme`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** `grow`, `preventGrowOverflow`

### SimpleGrid — M:SimpleGrid

来源：[@mantine/core](https://mantine.dev/core/simple-grid/)；对应 `P3-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoFlow`, `autoRows`, `cols`, `minColWidth`, `spacing`, `type`, `verticalSpacing`

**官方功能章节：** Usage；spacing and verticalSpacing props；Responsive props；Container queries；Auto-fill columns；auto-fill vs auto-fit；Auto rows；Browser support

**本地实现：** [../glory/crates/style/src/grid.rs](../../glory/crates/style/src/grid.rs)、[src/grid_layout.rs](../src/grid_layout.rs)、[src/responsive.rs](../src/responsive.rs)

**本地公开入口（同族）：** `aspect_ratio`, `breakpoint`, `breakpoints`, `class`, `column`, `column_tracks`, `columns`, `content`, `context_ref`, `fill`, `gap`, `grid`, `grid_item`, `height`, `item`, `items`, `name`, `new`, `node_ref`, `pending_queries`, `resolve`, `responsive_root`, `row`, `row_span`, `row_tracks`, `select`, `simple_grid`, `size`, `span`, `style`, `styles`, `theme`, `thresholds`, `unstyled`, `width`

**需要语义映射的属性候选：** `autoFlow`, `autoRows`, `cols`, `minColWidth`, `spacing`, `type`, `verticalSpacing`

### Space — M:Space

来源：[@mantine/core](https://mantine.dev/core/space/)；对应 `P3-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Usage；Where to use

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)、[src/responsive.rs](../src/responsive.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `breakpoint`, `breakpoints`, `class`, `container`, `content`, `context_ref`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pending_queries`, `pointer_events`, `position`, `radius`, `resolve`, `responsive_root`, `right`, `scroll_anchor`, `select`, `size`, `style`, `styles`, `text_overflow`, `theme`, `thresholds`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Splitter — M:Splitter

来源：[@mantine/core](https://mantine.dev/core/splitter/)；对应 `P6-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `handleColor`, `handleIcon`, `lineSize`, `onCollapseChange`, `onResizeEnd`, `onResizeStart`, `onSizeChange`, `orientation`, `redistribute`, `resetOnDoubleClick`, `shiftStep`, `sizes`, `splitterRef`, `step`, `withHandle`

**官方功能章节：** Usage；CSS units；Vertical orientation；Collapsible panes；Controlled；Multiple panes；Nested splitters；Redistribute；Line size；Without handle；Imperative API；Wrap Splitter.Pane；use-splitter hook

**本地实现：** [../glory/crates/render/src/range.rs](../../glory/crates/render/src/range.rs)、[src/splitter.rs](../src/splitter.rs)

**本地公开入口（同族）：** `axis`, `class`, `default_value`, `enabled`, `first`, `first_ref`, `focus`, `fraction`, `handle_ref`, `max`, `min`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `second`, `second_ref`, `size`, `splitter`, `step`, `style`, `theme`, `thickness`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `children`, `handleColor`, `handleIcon`, `lineSize`, `onCollapseChange`, `onResizeEnd`, `onResizeStart`, `onSizeChange`, `orientation`, `redistribute`, `resetOnDoubleClick`, `shiftStep`, `sizes`, `splitterRef`, `withHandle`

### Stack — M:Stack

来源：[@mantine/core](https://mantine.dev/core/stack/)；对应 `P3-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `align`, `gap`, `justify`

**官方功能章节：** Usage；Browser support

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `class`, `container`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pointer_events`, `position`, `radius`, `right`, `scroll_anchor`, `size`, `style`, `text_overflow`, `theme`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### AlphaSlider — M:AlphaSlider

来源：[@mantine/core](https://mantine.dev/core/alpha-slider/)；对应 `P7-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `focusable`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`, `size`, `value`

**官方功能章节：** Usage

**本地实现：** [src/color_model.rs](../src/color_model.rs)、[src/color_slider.rs](../src/color_slider.rs)、[src/slider/mod.rs](../src/slider/mod.rs)

**本地公开入口（同族）：** `alpha_slider`, `class`, `color`, `default_value`, `direction`, `enabled`, `field`, `format_hex_color`, `from_rgba`, `hue_slider`, `invalid`, `max`, `min`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `orientation`, `page_step`, `parse_hex_color`, `readonly`, `required`, `size`, `slider`, `step`, `style`, `theme`, `to_rgba`, `unstyled`, `validate`, `value`, `value_text`

**需要语义映射的属性候选：** `focusable`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`

### AngleSlider — M:AngleSlider

来源：[@mantine/core](https://mantine.dev/core/angle-slider/)；对应 `P7-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `defaultValue`, `disabled`, `formatLabel`, `hiddenInputProps`, `marks`, `name`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`, `restrictToMarks`, `size`, `step`, `thumbSize`, `value`, `withLabel`

**官方功能章节：** Usage；Controlled；AngleSlider with uncontrolled forms；formatLabel；Marks；onChangeEnd；disabled；Accessibility；Based on use-radial-move

**本地实现：** [src/angle_slider.rs](../src/angle_slider.rs)、[src/slider/mapping.rs](../src/slider/mapping.rs)、[src/slider/mod.rs](../src/slider/mod.rs)

**本地公开入口（同族）：** `angle_slider`, `class`, `default_value`, `direction`, `enabled`, `field`, `invalid`, `key`, `max`, `min`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `orientation`, `page_step`, `point`, `readonly`, `required`, `retains_increment`, `size`, `slider`, `step`, `style`, `theme`, `unstyled`, `validate`, `value`, `value_text`

**需要语义映射的属性候选：** `disabled`, `formatLabel`, `hiddenInputProps`, `marks`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`, `restrictToMarks`, `thumbSize`, `withLabel`

### Checkbox — M:Checkbox

来源：[@mantine/core](https://mantine.dev/core/checkbox/)；对应 `P0-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `checked`, `color`, `defaultChecked`, `description`, `disabled`, `error`, `icon`, `iconColor`, `id`, `indeterminate`, `label`, `labelPosition`, `onChange`, `radius`, `rootRef`, `size`, `value`, `withBorder`, `withErrorStyles`, `wrapperProps`

**官方功能章节：** Usage；Controlled state；Read only；Checkbox with @mantine/form；Checkbox with uncontrolled forms；States；Light variant；Error state；Change icons；Change icon color；Indeterminate state；Label with link；Checkbox with tooltip；Pointer cursor；Add custom sizes；Wrapper props；Checkbox.Group；Checkbox.Group disabled；maxSelectedValues；Checkbox.Group with @mantine/form；Checkbox.Group with uncontrolled forms；Checkbox.Indicator；Example: Table with row selection；Example: Customize with Styles API；wrapperProps；id attribute；Accessibility

**本地实现：** [src/checkbox.rs](../src/checkbox.rs)

**本地公开入口（同族）：** `checkbox`, `checked`, `class`, `default_checked`, `enabled`, `field`, `focus`, `invalid`, `label`, `name`, `node_ref`, `on`, `on_checked_change`, `readonly`, `required`, `size`, `style`, `theme`, `unstyled`

**需要语义映射的属性候选：** `autoContrast`, `color`, `description`, `disabled`, `error`, `icon`, `iconColor`, `id`, `indeterminate`, `labelPosition`, `onChange`, `radius`, `rootRef`, `value`, `withBorder`, `withErrorStyles`, `wrapperProps`

### Chip — M:Chip

来源：[@mantine/core](https://mantine.dev/core/chip/)；对应 `P2-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `checked`, `children`, `color`, `defaultChecked`, `defaultValue`, `icon`, `id`, `multiple`, `onChange`, `radius`, `rootRef`, `size`, `type`, `value`, `wrapperProps`

**官方功能章节：** Usage；Controlled；Read only；Change checked icon；States；Chip with tooltip；Wrapper props；Chip.Group；Controlled Chip.Group；Deselect radio chip；Accessibility

**本地实现：** [src/chip.rs](../src/chip.rs)

**本地公开入口（同族）：** `checked`, `chip`, `chip_group`, `chip_item`, `class`, `default_checked`, `default_value`, `default_values`, `enabled`, `field`, `focus`, `invalid`, `item`, `items`, `label`, `mode`, `name`, `node_ref`, `on`, `on_checked_change`, `on_value_change`, `on_values_change`, `readonly`, `required`, `size`, `style`, `theme`, `unstyled`, `value`, `values`

**需要语义映射的属性候选：** `autoContrast`, `children`, `color`, `icon`, `id`, `multiple`, `onChange`, `radius`, `rootRef`, `type`, `wrapperProps`

### ColorInput — M:ColorInput

来源：[@mantine/core](https://mantine.dev/core/color-input/)；对应 `P7-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `closeOnColorSwatchClick`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `disallowInput`, `error`, `errorProps`, `eyeDropperButtonProps`, `eyeDropperIcon`, `fixOnBlur`, `format`, `fullWidth`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `onChange`, `onChangeEnd`, `pointer`, `popoverProps`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `swatches`, `swatchesPerRow`, `value`, `withAsterisk`, `withErrorStyles`, `withEyeDropper`, `withPicker`, `withPreview`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Formats；Preserve invalid input；onChangeEnd；Disable free input；With swatches；Close dropdown on color swatch click；Hide dropdown；Eye dropper；Change eye dropper icon；Input sections；Error state；Success state；Disabled state

**本地实现：** [../glory/crates/render/src/drag.rs](../../glory/crates/render/src/drag.rs)、[src/color_input.rs](../src/color_input.rs)、[src/color_picker/mod.rs](../src/color_picker/mod.rs)、[src/color_picker/pad.rs](../src/color_picker/pad.rs)

**本地公开入口（同族）：** `as_str`, `class`, `color_input`, `color_picker`, `default_open`, `default_value`, `direction`, `enabled`, `event`, `field`, `invalid`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_open_change`, `on_validation`, `on_value_change`, `on_value_commit`, `open`, `plane_size`, `readonly`, `required`, `show_alpha`, `show_picker`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `closeOnColorSwatchClick`, `description`, `descriptionProps`, `disabled`, `disallowInput`, `error`, `errorProps`, `eyeDropperButtonProps`, `eyeDropperIcon`, `fixOnBlur`, `format`, `fullWidth`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `onChange`, `onChangeEnd`, `pointer`, `popoverProps`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `swatches`, `swatchesPerRow`, `withAsterisk`, `withErrorStyles`, `withEyeDropper`, `withPicker`, `withPreview`, `withSuccessStyles`, `wrapperProps`

### ColorPicker — M:ColorPicker

来源：[@mantine/core](https://mantine.dev/core/color-picker/)；对应 `P7-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `alphaLabel`, `defaultValue`, `focusable`, `format`, `fullWidth`, `hiddenInputProps`, `hueLabel`, `name`, `onChange`, `onChangeEnd`, `onColorSwatchClick`, `saturationLabel`, `size`, `swatches`, `swatchesPerRow`, `value`, `withPicker`

**官方功能章节：** Usage；Controlled；Uncontrolled；Color format；With swatches；Size；Accessibility

**本地实现：** [../glory/crates/render/src/drag.rs](../../glory/crates/render/src/drag.rs)、[src/color_input.rs](../src/color_input.rs)、[src/color_picker/mod.rs](../src/color_picker/mod.rs)、[src/color_picker/pad.rs](../src/color_picker/pad.rs)

**本地公开入口（同族）：** `as_str`, `class`, `color_input`, `color_picker`, `default_open`, `default_value`, `direction`, `enabled`, `event`, `field`, `invalid`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_open_change`, `on_validation`, `on_value_change`, `on_value_commit`, `open`, `plane_size`, `readonly`, `required`, `show_alpha`, `show_picker`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `alphaLabel`, `focusable`, `format`, `fullWidth`, `hiddenInputProps`, `hueLabel`, `onChange`, `onChangeEnd`, `onColorSwatchClick`, `saturationLabel`, `swatches`, `swatchesPerRow`, `withPicker`

### Fieldset — M:Fieldset

来源：[@mantine/core](https://mantine.dev/core/fieldset/)；对应 `P1-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `legend`, `radius`

**官方功能章节：** Usage；Disabled

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/blitz/src/shell.rs](../../glory/crates/blitz/src/shell.rs)、[../glory/crates/html/src/semantics.rs](../../glory/crates/html/src/semantics.rs)、[../glory/crates/render/src/semantics.rs](../../glory/crates/render/src/semantics.rs)、[../glory/crates/web/src/browser/events.rs](../../glory/crates/web/src/browser/events.rs)、[src/checkbox.rs](../src/checkbox.rs)、[src/field.rs](../src/field.rs)、[src/text_field.rs](../src/text_field.rs)

**本地公开入口（同族）：** `add_window`, `android`, `app`, `as_str`, `checkbox`, `checked`, `class`, `content`, `control`, `default_checked`, `default_value`, `description`, `disabled`, `enabled`, `error`, `field`, `field_description`, `field_error`, `fieldset`, `fill`, `focus`, `form_name`, `form_value`, `frames`, `from_event_loop`, `handle_native_file_event`, `invalid`, `key`, `key_with_direction`, `label`, `legend`, `mount`, `mount_with`, `multiline`, `name`, `new`, `node_ref`, `observe`, `on`, `on_checked_change`, `on_commit`, `on_value_change`, `placeholder`, `purpose`, `readonly`, `renderer`, `request_size`, `required`, `run`, `size`, `style`, `take_event_loop`, `text_field`, `theme`, `unstyled`, `value`, `window_stats`, `with_config`, `with_limit`, `with_options`, `with_task_limit`

**需要语义映射的属性候选：** `radius`

### FileInput — M:FileInput

来源：[@mantine/core](https://mantine.dev/core/file-input/)；对应 `P10-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accept`, `capture`, `clearButtonProps`, `clearSectionMode`, `clearable`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `fileInputProps`, `form`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `multiple`, `name`, `onChange`, `placeholder`, `pointer`, `radius`, `readOnly`, `required`, `resetRef`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `value`, `valueComponent`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Multiple；Accept；Clearable；Custom value component；Error state；Success state；FileInputProps type

**本地实现：** [../glory/crates/render/src/files.rs](../../glory/crates/render/src/files.rs)、[../glory/crates/web/src/browser/files.rs](../../glory/crates/web/src/browser/files.rs)、[src/files.rs](../src/files.rs)

**本地公开入口（同族）：** `attachment`, `class`, `clear`, `configuration`, `controller_ref`, `description`, `dropzone`, `enabled`, `file_attachment`, `file_button`, `file_input`, `files`, `files_ref`, `focus`, `label`, `matches`, `name`, `native_source`, `new`, `node_ref`, `on`, `on_files`, `on_result`, `preview`, `preview_in`, `previews`, `read`, `read_in`, `remove`, `size`, `status`, `status_ref`, `style`, `text`, `theme`, `unstyled`, `validate`, `validate_file`, `validate_files`, `validate_node`, `validate_read`, `validate_update`, `value`

**需要语义映射的属性候选：** `accept`, `capture`, `clearButtonProps`, `clearSectionMode`, `clearable`, `defaultValue`, `descriptionProps`, `disabled`, `error`, `errorProps`, `fileInputProps`, `form`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `multiple`, `onChange`, `placeholder`, `pointer`, `radius`, `readOnly`, `required`, `resetRef`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `valueComponent`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### HueSlider — M:HueSlider

来源：[@mantine/core](https://mantine.dev/core/hue-slider/)；对应 `P7-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `focusable`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`, `size`, `value`

**官方功能章节：** Usage

**本地实现：** [src/color_model.rs](../src/color_model.rs)、[src/color_slider.rs](../src/color_slider.rs)、[src/slider/mod.rs](../src/slider/mod.rs)

**本地公开入口（同族）：** `alpha_slider`, `class`, `color`, `default_value`, `direction`, `enabled`, `field`, `format_hex_color`, `from_rgba`, `hue_slider`, `invalid`, `max`, `min`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `orientation`, `page_step`, `parse_hex_color`, `readonly`, `required`, `size`, `slider`, `step`, `style`, `theme`, `to_rgba`, `unstyled`, `validate`, `value`, `value_text`

**需要语义映射的属性候选：** `focusable`, `onChange`, `onChangeEnd`, `onScrubEnd`, `onScrubStart`

### Input — M:Input

来源：[@mantine/core](https://mantine.dev/core/input/)；对应 `P1-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `id`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelElement`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `multiline`, `pointer`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `size`, `success`, `successProps`, `withAria`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Disclaimer；Usage；Loading state；Input sections；Change input element；Input.Wrapper component；inputWrapperOrder；inputContainer；required and withAsterisk props；error prop；success prop；Input.Label, Input.Description and Input.Error components；Input.Placeholder component；Input.ClearButton component；Default props on theme；Shared default props for all inputs；Styles on theme；Change focus styles；InputBase component；Accessibility

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/blitz/src/shell.rs](../../glory/crates/blitz/src/shell.rs)、[../glory/crates/html/src/semantics.rs](../../glory/crates/html/src/semantics.rs)、[../glory/crates/render/src/semantics.rs](../../glory/crates/render/src/semantics.rs)、[../glory/crates/web/src/browser/events.rs](../../glory/crates/web/src/browser/events.rs)、[src/checkbox.rs](../src/checkbox.rs)、[src/field.rs](../src/field.rs)、[src/text_field.rs](../src/text_field.rs)

**本地公开入口（同族）：** `add_window`, `android`, `app`, `as_str`, `checkbox`, `checked`, `class`, `content`, `control`, `default_checked`, `default_value`, `description`, `disabled`, `enabled`, `error`, `field`, `field_description`, `field_error`, `fieldset`, `fill`, `focus`, `form_name`, `form_value`, `frames`, `from_event_loop`, `handle_native_file_event`, `invalid`, `key`, `key_with_direction`, `label`, `legend`, `mount`, `mount_with`, `multiline`, `name`, `new`, `node_ref`, `observe`, `on`, `on_checked_change`, `on_commit`, `on_value_change`, `placeholder`, `purpose`, `readonly`, `renderer`, `request_size`, `required`, `run`, `size`, `style`, `take_event_loop`, `text_field`, `theme`, `unstyled`, `value`, `window_stats`, `with_config`, `with_limit`, `with_options`, `with_task_limit`

**需要语义映射的属性候选：** `descriptionProps`, `errorProps`, `id`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelElement`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `success`, `successProps`, `withAria`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### JsonInput — M:JsonInput

来源：[@mantine/core](https://mantine.dev/core/json-input/)；对应 `P7-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autosize`, `bottomSection`, `bottomSectionProps`, `defaultValue`, `description`, `descriptionProps`, `deserialize`, `disabled`, `error`, `errorProps`, `formatOnBlur`, `indentSpaces`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxRows`, `minRows`, `onChange`, `pointer`, `radius`, `required`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `serialize`, `size`, `success`, `successProps`, `validationError`, `value`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Custom serialization；Input props；Success state

**本地实现：** [src/json_input.rs](../src/json_input.rs)、[src/json_model.rs](../src/json_model.rs)

**本地公开入口（同族）：** `announce_errors`, `class`, `default_value`, `enabled`, `field`, `format`, `format_json`, `format_label`, `invalid`, `json_input`, `limits`, `name`, `node_ref`, `on`, `on_commit`, `on_format`, `on_format_error`, `on_validation`, `on_value_change`, `placeholder`, `readonly`, `required`, `rows`, `show_formatter`, `size`, `style`, `theme`, `unstyled`, `validate`, `validate_json`, `value`

**需要语义映射的属性候选：** `autosize`, `bottomSection`, `bottomSectionProps`, `description`, `descriptionProps`, `deserialize`, `disabled`, `error`, `errorProps`, `formatOnBlur`, `indentSpaces`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxRows`, `minRows`, `onChange`, `pointer`, `radius`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `serialize`, `success`, `successProps`, `validationError`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### MaskInput — M:MaskInput

来源：[@mantine/core](https://mantine.dev/core/mask-input/)；对应 `P7-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `alwaysShowMask`, `autoClear`, `beforeMaskedStateChange`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `mask`, `modify`, `onChangeRaw`, `onComplete`, `pointer`, `radius`, `required`, `resetRef`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `separate`, `showMaskOnFocus`, `size`, `slotChar`, `success`, `successProps`, `tokens`, `transform`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Dynamic mask；Custom tokens；Regex array format；Transform；Disabled state；Error state；Success state；Reset value；With use-form；Mask pattern syntax

**本地实现：** [../glory/crates/render/src/input_edit.rs](../../glory/crates/render/src/input_edit.rs)、[../glory/crates/render/src/text_selection.rs](../../glory/crates/render/src/text_selection.rs)、[../glory/crates/web/src/browser/input_edit.rs](../../glory/crates/web/src/browser/input_edit.rs)、[src/mask_input.rs](../src/mask_input.rs)、[src/mask_model.rs](../src/mask_model.rs)

**本地公开入口（同族）：** `apply`, `as_str`, `byte_range`, `capacity`, `class`, `default_value`, `display`, `display_index`, `enabled`, `field`, `focus`, `form_name`, `format`, `invalid`, `mask_input`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_reject`, `on_value_change`, `parse`, `parse_display`, `raw`, `raw_index`, `readonly`, `required`, `revision`, `size`, `style`, `theme`, `unstyled`, `validate`, `validate_raw`, `value`

**需要语义映射的属性候选：** `alwaysShowMask`, `autoClear`, `beforeMaskedStateChange`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `mask`, `modify`, `onChangeRaw`, `onComplete`, `pointer`, `radius`, `resetRef`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `separate`, `showMaskOnFocus`, `slotChar`, `success`, `successProps`, `tokens`, `transform`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### NativeSelect — M:NativeSelect

来源：[@mantine/core](https://mantine.dev/core/native-select/)；对应 `P4-04`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `data`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Adding options；With dividers；Input sections；Disabled state；Error state

**本地实现：** [../glory/crates/render/src/select.rs](../../glory/crates/render/src/select.rs)、[src/native_select.rs](../src/native_select.rs)

**本地公开入口（同族）：** `class`, `default_value`, `default_values`, `disabled`, `enabled`, `field`, `focus`, `form_name`, `group`, `invalid`, `label`, `multiple`, `name`, `native_select`, `node_ref`, `on`, `on_value_change`, `on_values_change`, `option`, `options`, `placeholder`, `readonly`, `required`, `rows`, `select_group`, `select_option`, `size`, `style`, `theme`, `unstyled`, `validate`, `validate_selection`, `validate_user_selection`, `value`, `values`

**需要语义映射的属性候选：** `data`, `description`, `descriptionProps`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### NumberInput — M:NumberInput

来源：[@mantine/core](https://mantine.dev/core/number-input/)；对应 `P7-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDecimal`, `allowLeadingZeros`, `allowNegative`, `allowedDecimalSeparators`, `clampBehavior`, `decimalScale`, `decimalSeparator`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `fixedDecimalScale`, `handlersRef`, `hideControls`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `isAllowed`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `max`, `min`, `onChange`, `onMaxReached`, `onMinReached`, `onValueChange`, `prefix`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `selectAllOnFocus`, `size`, `startValue`, `step`, `stepHoldDelay`, `stepHoldInterval`, `success`, `successProps`, `suffix`, `thousandSeparator`, `thousandsGroupStyle`, `trimLeadingZeroesOnBlur`, `type`, `value`, `valueIsNumericString`, `withAsterisk`, `withErrorStyles`, `withKeyboardEvents`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Value type；BigInt values；onChange vs onValueChange；min and max；Clamp behavior；Boundary callbacks；Select all on focus；Prefix and suffix；Negative numbers；Decimal numbers；Decimal scale；Fixed decimal scale；Decimal separator；Thousand separator；Trim leading zeros on blur；Input sections；Increment/decrement controls；Increment/decrement on hold；Custom increment and decrement controls；Error state；Success state

**本地实现：** [../glory/crates/render/src/spinbutton.rs](../../glory/crates/render/src/spinbutton.rs)、[src/number_input.rs](../src/number_input.rs)、[src/number_model.rs](../src/number_model.rs)

**本地公开入口（同族）：** `clamp`, `class`, `coefficient`, `controls`, `default_value`, `enabled`, `field`, `focus`, `form_name`, `format`, `grouping`, `increment`, `invalid`, `locale`, `max`, `min`, `name`, `new`, `node_ref`, `number_input`, `on`, `on_commit`, `on_reject`, `on_value_change`, `outside`, `page_step`, `parse`, `parse_number`, `readonly`, `required`, `scale`, `size`, `spin_key`, `step`, `style`, `text`, `theme`, `to_f64`, `unstyled`, `validate`, `validate_command`, `value`

**需要语义映射的属性候选：** `allowDecimal`, `allowLeadingZeros`, `allowNegative`, `allowedDecimalSeparators`, `clampBehavior`, `decimalScale`, `decimalSeparator`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `fixedDecimalScale`, `handlersRef`, `hideControls`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `isAllowed`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `onChange`, `onMaxReached`, `onMinReached`, `prefix`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `selectAllOnFocus`, `startValue`, `stepHoldDelay`, `stepHoldInterval`, `success`, `successProps`, `suffix`, `thousandSeparator`, `thousandsGroupStyle`, `trimLeadingZeroesOnBlur`, `type`, `valueIsNumericString`, `withAsterisk`, `withErrorStyles`, `withKeyboardEvents`, `withSuccessStyles`, `wrapperProps`

### PasswordInput — M:PasswordInput

来源：[@mantine/core](https://mantine.dev/core/password-input/)；对应 `P1-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `defaultVisible`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `onVisibilityChange`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `visibilityToggleButtonProps`, `visibilityToggleFocusable`, `visibilityToggleIcon`, `visible`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Controlled visibility toggle；Change visibility toggle icon；Strength meter example；Usage without visibility toggle；Input sections；Error state；Success state

**本地实现：** [src/password_input.rs](../src/password_input.rs)、[src/text_field.rs](../src/text_field.rs)、[src/text_input.rs](../src/text_input.rs)

**本地公开入口（同族）：** `class`, `default_value`, `default_visible`, `enabled`, `field`, `focus`, `invalid`, `multiline`, `name`, `node_ref`, `on`, `on_commit`, `on_value_change`, `on_visibility_change`, `password_input`, `placeholder`, `purpose`, `readonly`, `required`, `size`, `style`, `text_field`, `theme`, `unstyled`, `value`, `visibility_labels`, `visible`

**需要语义映射的属性候选：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `visibilityToggleButtonProps`, `visibilityToggleFocusable`, `visibilityToggleIcon`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### PinInput — M:PinInput

来源：[@mantine/core](https://mantine.dev/core/pin-input/)；对应 `P7-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `ariaLabel`, `autoFocus`, `defaultValue`, `disabled`, `error`, `form`, `gap`, `getInputProps`, `hiddenInputProps`, `id`, `inputMode`, `inputType`, `length`, `manageFocus`, `mask`, `name`, `onChange`, `onComplete`, `oneTimeCode`, `placeholder`, `radius`, `readOnly`, `rootRef`, `size`, `success`, `type`, `value`

**官方功能章节：** Usage；Controlled；Uncontrolled；Regex type；One time code；Accessibility

**本地实现：** [../glory/crates/render/src/input_assist.rs](../../glory/crates/render/src/input_assist.rs)、[src/pin_input.rs](../src/pin_input.rs)、[src/pin_model.rs](../src/pin_model.rs)

**本地公开入口（同族）：** `alphabet`, `as_str`, `autofill`, `class`, `complete`, `completed_user_transition`, `default_value`, `enabled`, `field`, `focus`, `form_name`, `input_otp`, `invalid`, `length`, `masked`, `name`, `node_ref`, `on`, `on_commit`, `on_complete`, `on_reject`, `on_value_change`, `pin_input`, `readonly`, `required`, `size`, `style`, `theme`, `unstyled`, `validate`, `validate_input_placeholder`, `validate_value`, `value`

**需要语义映射的属性候选：** `ariaLabel`, `autoFocus`, `disabled`, `error`, `form`, `gap`, `getInputProps`, `hiddenInputProps`, `id`, `inputMode`, `inputType`, `manageFocus`, `mask`, `onChange`, `oneTimeCode`, `placeholder`, `radius`, `readOnly`, `rootRef`, `success`, `type`

### Radio — M:Radio

来源：[@mantine/core](https://mantine.dev/core/radio/)；对应 `P2-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `checked`, `children`, `color`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `icon`, `iconColor`, `id`, `inputContainer`, `inputWrapperOrder`, `label`, `labelElement`, `labelPosition`, `labelProps`, `name`, `onChange`, `radius`, `readOnly`, `required`, `rootRef`, `size`, `success`, `successProps`, `value`, `withAsterisk`, `withBorder`, `withErrorStyles`, `wrapperProps`

**官方功能章节：** Usage；Controlled；Uncontrolled；Read only；States；Light variant；Change icon；Change icon color；Disabled state；Pointer cursor；Radio with tooltip；Wrapper props；Radio.Group component；Radio.Group disabled state；Controlled Radio.Group；Radio.Indicator；Accessibility

**本地实现：** [../glory/crates/html/src/patch.rs](../../glory/crates/html/src/patch.rs)、[../glory/crates/render/src/props.rs](../../glory/crates/render/src/props.rs)、[src/checkbox_group.rs](../src/checkbox_group.rs)、[src/mixed_checkbox.rs](../src/mixed_checkbox.rs)、[src/radio.rs](../src/radio.rs)、[src/segmented_control.rs](../src/segmented_control.rs)、[src/switch.rs](../src/switch.rs)

**本地公开入口（同族）：** `accepts_children`, `activation_enabled`, `checkbox_group`, `checkbox_item`, `checked`, `class`, `default_checked`, `default_selected`, `default_state`, `default_value`, `direction`, `document_part`, `effective_role`, `enabled`, `field`, `field_part`, `focus`, `form_name`, `get`, `hidden`, `insert`, `interactive`, `invalid`, `item`, `items`, `label`, `loop_focus`, `mixed_checkbox`, `name`, `new`, `node_ref`, `normalize_input`, `on`, `on_checked_change`, `on_selected_change`, `on_state_change`, `on_value_change`, `orientation`, `patch`, `patched`, `patches`, `radio`, `radio_group`, `readonly`, `required`, `segment`, `segmented_control`, `segments`, `select_all_label`, `selected`, `semantic_id`, `size`, `state`, `style`, `switch`, `theme`, `typeahead`, `unstyled`, `validate`, `value`, `view`

**需要语义映射的属性候选：** `autoContrast`, `children`, `color`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `icon`, `iconColor`, `id`, `inputContainer`, `inputWrapperOrder`, `labelElement`, `labelPosition`, `labelProps`, `onChange`, `radius`, `readOnly`, `rootRef`, `success`, `successProps`, `withAsterisk`, `withBorder`, `withErrorStyles`, `wrapperProps`

### RangeSlider — M:RangeSlider

来源：[@mantine/core](https://mantine.dev/core/range-slider/)；对应 `P7-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `defaultValue`, `disabled`, `domain`, `hiddenInputProps`, `inverted`, `label`, `labelAlwaysOn`, `labelTransitionProps`, `marks`, `max`, `maxRange`, `min`, `minRange`, `name`, `onChange`, `onChangeEnd`, `orientation`, `precision`, `pushOnOverlap`, `radius`, `restrictToMarks`, `scale`, `showLabelOnHover`, `size`, `step`, `thumbChildren`, `thumbFromLabel`, `thumbLabel`, `thumbProps`, `thumbSize`, `thumbToLabel`, `thumbValueText`, `value`

**官方功能章节：** Usage；Controlled；Uncontrolled；Disabled；Control label；Min, max and step；Domain；Decimal values；minRange；maxRange；pushOnOverlap；Marks；Restrict selection to marks；Inverted；Hidden marks；Accessibility；Keyboard interactions

**本地实现：** [src/range_slider.rs](../src/range_slider.rs)

**本地公开入口（同族）：** `class`, `collision`, `default_value`, `direction`, `enabled`, `field`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `orientation`, `page_step`, `range_slider`, `size`, `style`, `theme`, `thumb_ref`, `unstyled`, `value`

**需要语义映射的属性候选：** `color`, `disabled`, `domain`, `hiddenInputProps`, `inverted`, `label`, `labelAlwaysOn`, `labelTransitionProps`, `marks`, `max`, `maxRange`, `min`, `minRange`, `onChange`, `onChangeEnd`, `precision`, `pushOnOverlap`, `radius`, `restrictToMarks`, `scale`, `showLabelOnHover`, `step`, `thumbChildren`, `thumbFromLabel`, `thumbLabel`, `thumbProps`, `thumbSize`, `thumbToLabel`, `thumbValueText`

### Rating — M:Rating

来源：[@mantine/core](https://mantine.dev/core/rating/)；对应 `P7-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowClear`, `color`, `count`, `defaultValue`, `emptySymbol`, `fractions`, `fullSymbol`, `getSymbolLabel`, `highlightSelectedOnly`, `name`, `onChange`, `onHover`, `readOnly`, `size`, `value`

**官方功能章节：** Usage；Controlled；Uncontrolled；Read only；Allow clear；Fractions；Custom symbol；Symbols for each item

**本地实现：** [src/radio.rs](../src/radio.rs)、[src/rating/mod.rs](../src/rating/mod.rs)、[src/rating/model.rs](../src/rating/model.rs)

**本地公开入口（同族）：** `class`, `clearable`, `count`, `default_value`, `direction`, `display_only`, `enabled`, `field`, `focus`, `form_name`, `invalid`, `items`, `label`, `loop_focus`, `name`, `node_ref`, `on`, `on_value_change`, `option_count`, `orientation`, `precision`, `radio`, `radio_group`, `rating`, `readonly`, `required`, `size`, `style`, `theme`, `typeahead`, `units`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allowClear`, `color`, `emptySymbol`, `fractions`, `fullSymbol`, `getSymbolLabel`, `highlightSelectedOnly`, `onChange`, `onHover`, `readOnly`

### SegmentedControl — M:SegmentedControl

来源：[@mantine/core](https://mantine.dev/core/segmented-control/)；对应 `P2-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `color`, `data`, `defaultValue`, `disabled`, `fullWidth`, `name`, `onChange`, `orientation`, `radius`, `readOnly`, `size`, `transitionDuration`, `transitionTimingFunction`, `value`, `withItemsBorders`

**官方功能章节：** Usage；Controlled；Uncontrolled；Data prop；Generic value type；Disabled；React node as label；Color；Auto contrast；Transitions；Accessibility and usability

**本地实现：** [../glory/crates/html/src/patch.rs](../../glory/crates/html/src/patch.rs)、[../glory/crates/render/src/props.rs](../../glory/crates/render/src/props.rs)、[src/checkbox_group.rs](../src/checkbox_group.rs)、[src/mixed_checkbox.rs](../src/mixed_checkbox.rs)、[src/radio.rs](../src/radio.rs)、[src/segmented_control.rs](../src/segmented_control.rs)、[src/switch.rs](../src/switch.rs)

**本地公开入口（同族）：** `accepts_children`, `activation_enabled`, `checkbox_group`, `checkbox_item`, `checked`, `class`, `default_checked`, `default_selected`, `default_state`, `default_value`, `direction`, `document_part`, `effective_role`, `enabled`, `field`, `field_part`, `focus`, `form_name`, `get`, `hidden`, `insert`, `interactive`, `invalid`, `item`, `items`, `label`, `loop_focus`, `mixed_checkbox`, `name`, `new`, `node_ref`, `normalize_input`, `on`, `on_checked_change`, `on_selected_change`, `on_state_change`, `on_value_change`, `orientation`, `patch`, `patched`, `patches`, `radio`, `radio_group`, `readonly`, `required`, `segment`, `segmented_control`, `segments`, `select_all_label`, `selected`, `semantic_id`, `size`, `state`, `style`, `switch`, `theme`, `typeahead`, `unstyled`, `validate`, `value`, `view`

**需要语义映射的属性候选：** `autoContrast`, `color`, `data`, `disabled`, `fullWidth`, `onChange`, `radius`, `readOnly`, `transitionDuration`, `transitionTimingFunction`, `withItemsBorders`

### Slider — M:Slider

来源：[@mantine/core](https://mantine.dev/core/slider/)；对应 `P7-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `defaultValue`, `disabled`, `domain`, `hiddenInputProps`, `inverted`, `label`, `labelAlwaysOn`, `labelTransitionProps`, `marks`, `max`, `min`, `name`, `onChange`, `onChangeEnd`, `orientation`, `precision`, `radius`, `restrictToMarks`, `scale`, `showLabelOnHover`, `size`, `startPointValue`, `step`, `thumbChildren`, `thumbLabel`, `thumbProps`, `thumbSize`, `thumbValueText`, `value`

**官方功能章节：** Usage；Controlled；Uncontrolled；Disabled；onChangeEnd；Control label；Min, max and step；Domain；Decimal values；Marks；Restrict selection to marks；Thumb size；Thumb children；Scale；Start point；Vertical slider；Hidden marks；Build custom slider；Accessibility；Keyboard interactions

**本地实现：** [src/slider/mod.rs](../src/slider/mod.rs)

**本地公开入口（同族）：** `class`, `default_value`, `direction`, `enabled`, `field`, `invalid`, `max`, `min`, `name`, `node_ref`, `on`, `on_value_change`, `on_value_commit`, `orientation`, `page_step`, `readonly`, `required`, `size`, `slider`, `step`, `style`, `theme`, `unstyled`, `value`, `value_text`

**需要语义映射的属性候选：** `color`, `disabled`, `domain`, `hiddenInputProps`, `inverted`, `label`, `labelAlwaysOn`, `labelTransitionProps`, `marks`, `onChange`, `onChangeEnd`, `precision`, `radius`, `restrictToMarks`, `scale`, `showLabelOnHover`, `startPointValue`, `thumbChildren`, `thumbLabel`, `thumbProps`, `thumbSize`, `thumbValueText`

### Switch — M:Switch

来源：[@mantine/core](https://mantine.dev/core/switch/)；对应 `P2-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `description`, `error`, `id`, `label`, `labelPosition`, `offLabel`, `onLabel`, `radius`, `rootRef`, `size`, `thumbIcon`, `withThumbIndicator`, `wrapperProps`

**官方功能章节：** Usage；Controlled；Uncontrolled；Read only；States；Inner Labels；Icon labels；Thumb icon；With tooltip；Pointer cursor；Wrapper props；Switch.Group；Switch.Group with uncontrolled forms；maxSelectedValues；Switch.Group disabled；Controlled Switch.Group；Get input ref；Accessibility

**本地实现：** [../glory/crates/html/src/patch.rs](../../glory/crates/html/src/patch.rs)、[../glory/crates/render/src/props.rs](../../glory/crates/render/src/props.rs)、[src/checkbox_group.rs](../src/checkbox_group.rs)、[src/mixed_checkbox.rs](../src/mixed_checkbox.rs)、[src/radio.rs](../src/radio.rs)、[src/segmented_control.rs](../src/segmented_control.rs)、[src/switch.rs](../src/switch.rs)

**本地公开入口（同族）：** `accepts_children`, `activation_enabled`, `checkbox_group`, `checkbox_item`, `checked`, `class`, `default_checked`, `default_selected`, `default_state`, `default_value`, `direction`, `document_part`, `effective_role`, `enabled`, `field`, `field_part`, `focus`, `form_name`, `get`, `hidden`, `insert`, `interactive`, `invalid`, `item`, `items`, `label`, `loop_focus`, `mixed_checkbox`, `name`, `new`, `node_ref`, `normalize_input`, `on`, `on_checked_change`, `on_selected_change`, `on_state_change`, `on_value_change`, `orientation`, `patch`, `patched`, `patches`, `radio`, `radio_group`, `readonly`, `required`, `segment`, `segmented_control`, `segments`, `select_all_label`, `selected`, `semantic_id`, `size`, `state`, `style`, `switch`, `theme`, `typeahead`, `unstyled`, `validate`, `value`, `view`

**需要语义映射的属性候选：** `color`, `description`, `error`, `id`, `labelPosition`, `offLabel`, `onLabel`, `radius`, `rootRef`, `thumbIcon`, `withThumbIndicator`, `wrapperProps`

### Textarea — M:Textarea

来源：[@mantine/core](https://mantine.dev/core/textarea/)；对应 `P1-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autosize`, `bottomSection`, `bottomSectionProps`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxRows`, `minRows`, `pointer`, `radius`, `required`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Bottom section；Autosize；Enable resize；Error state；Success state

**本地实现：** [../glory/crates/blitz/src/document.rs](../../glory/crates/blitz/src/document.rs)、[../glory/crates/html/src/styles.rs](../../glory/crates/html/src/styles.rs)、[../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/text_field.rs](../src/text_field.rs)、[src/text_input.rs](../src/text_input.rs)、[src/textarea.rs](../src/textarea.rs)

**本地公开入口（同族）：** `advance_time`, `aspect_ratio`, `autosize`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `class`, `default_value`, `display`, `enabled`, `engine_id`, `field`, `field_sizing`, `flex_direction`, `flex_wrap`, `focus`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `inspect`, `invalid`, `invalidate`, `is_finite`, `left`, `margin`, `max_height`, `max_rows`, `max_width`, `min_height`, `min_rows`, `min_width`, `multiline`, `name`, `new`, `node_ref`, `object_fit`, `observe_resources`, `on`, `on_commit`, `on_value_change`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `placeholder`, `pointer_events`, `poll_resources`, `position`, `profile`, `purpose`, `radius`, `readonly`, `required`, `resource_stats`, `right`, `rows`, `scroll_anchor`, `set_css`, `set_sheet`, `set_viewport`, `size`, `stats`, `style`, `text_field`, `text_overflow`, `textarea`, `theme`, `top`, `unstyled`, `validate`, `value`, `white_space`, `width`, `with_config`

**需要语义映射的属性候选：** `bottomSection`, `bottomSectionProps`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### TextInput — M:TextInput

来源：[@mantine/core](https://mantine.dev/core/text-input/)；对应 `P0-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Uncontrolled；Input sections；Error state；Success state

**本地实现：** [src/text_field.rs](../src/text_field.rs)

**本地公开入口（同族）：** `class`, `default_value`, `enabled`, `field`, `focus`, `invalid`, `multiline`, `name`, `node_ref`, `on`, `on_commit`, `on_value_change`, `placeholder`, `purpose`, `readonly`, `required`, `size`, `style`, `text_field`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### Autocomplete — M:Autocomplete

来源：[@mantine/core](https://mantine.dev/core/autocomplete/)；对应 `P4-07`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `autoSelectOnBlur`, `clearButtonProps`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `openOnFocus`, `radius`, `renderOption`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `size`, `success`, `successProps`, `value`, `withAsterisk`, `withErrorStyles`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Not a searchable select；Usage；Loading state；Controlled；Uncontrolled；Select first option on change；autoSelectOnBlur；Data prop；Filtering；Sort options；Fuzzy search with fuse.js；Large datasets；renderOption；Nothing found message；Scrollable dropdown；Fit dropdown to viewport height；Group options；Disabled options；Inside Popover；Clearable；Control dropdown opened state；Dropdown position；Dropdown animation；Dropdown padding；Dropdown shadow；Input sections；Input props；Read only；Disabled；Error state

**本地实现：** [src/autocomplete.rs](../src/autocomplete.rs)、[src/combobox.rs](../src/combobox.rs)

**本地公开入口（同族）：** `autocomplete`, `autocomplete_option`, `class`, `clear_all`, `combobox`, `default_open`, `default_search`, `default_value`, `empty_message`, `enabled`, `field`, `filter`, `focus`, `form_name`, `group`, `invalid`, `label`, `name`, `node_ref`, `on`, `on_commit`, `on_open_change`, `on_option_select`, `on_search_change`, `on_search_commit`, `on_value_change`, `open`, `option`, `options`, `placeholder`, `readonly`, `required`, `search`, `select_all`, `selected_key`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `autoSelectOnBlur`, `clearButtonProps`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `floatingHeight`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `openOnFocus`, `radius`, `renderOption`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

### Cascader — M:Cascader

来源：[@mantine/core](https://mantine.dev/core/cascader/)；对应 `P8-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `changeOnSelect`, `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnSelect`, `columnWidth`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `expandTrigger`, `filter`, `formatValue`, `hiddenInputProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxDisplayedLevels`, `maxDropdownHeight`, `nextLevelsControlLabel`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onSearchChange`, `openOnFocus`, `pointer`, `previousLevelsControlLabel`, `radius`, `renderOption`, `renderSearchOption`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `safeAreaPolygon`, `scrollAreaProps`, `searchValue`, `searchable`, `separator`, `size`, `success`, `successProps`, `value`, `withAsterisk`, `withCheckIcon`, `withColumns`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Data prop；Value and onChange；Change on select；Close on select；Allow deselect；Expand trigger；Flat list layout；Max displayed levels；Searchable；Controlled search value；Nothing found；Display value；Column width；Custom option rendering；Check icon；Disabled options；Left and right sections；Clearable；Scrollable columns；Control dropdown opened state；Dropdown position；Dropdown offset；Dropdown width；Dropdown padding；Dropdown shadow；Dropdown animation；Inside Popover；Input props；Read only；Disabled；Error state；Success state；Loading state；Accessibility；Keyboard interactions

**本地实现：** [../glory/crates/render/src/choice.rs](../../glory/crates/render/src/choice.rs)、[../glory/crates/render/src/tree.rs](../../glory/crates/render/src/tree.rs)、[src/cascader.rs](../src/cascader.rs)、[src/tree.rs](../src/tree.rs)、[src/tree_choice_model.rs](../src/tree_choice_model.rs)、[src/tree_select.rs](../src/tree_select.rs)

**本地公开入口（同族）：** `as_str`, `cascader`, `checked`, `choice`, `choice_remove_last`, `class`, `clearable`, `columns`, `contains`, `default_expanded`, `default_open`, `default_selected`, `default_value`, `direction`, `enabled`, `expanded`, `field`, `file_icons`, `file_tree`, `focus`, `form_name`, `height`, `invalid`, `is_empty`, `key`, `keys`, `leaf`, `multiple`, `name`, `new`, `node_ref`, `nodes`, `on`, `on_activate`, `on_expanded_change`, `on_load_children`, `on_open_change`, `on_selection_change`, `on_selection_intent`, `on_value_change`, `open`, `path`, `paths`, `placeholder`, `policy`, `readonly`, `render_label`, `required`, `resolve_active_descendant`, `selected`, `selection_follows_focus`, `selection_mode`, `size`, `style`, `theme`, `toggle`, `tree`, `tree_select`, `typeahead`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allowDeselect`, `changeOnSelect`, `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSectionMode`, `closeOnSelect`, `columnWidth`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `expandTrigger`, `filter`, `formatValue`, `hiddenInputProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxDisplayedLevels`, `maxDropdownHeight`, `nextLevelsControlLabel`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onSearchChange`, `openOnFocus`, `pointer`, `previousLevelsControlLabel`, `radius`, `renderOption`, `renderSearchOption`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `safeAreaPolygon`, `scrollAreaProps`, `searchValue`, `searchable`, `separator`, `success`, `successProps`, `withAsterisk`, `withCheckIcon`, `withColumns`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### Combobox — M:Combobox

来源：[@mantine/core](https://mantine.dev/core/combobox/)；对应 `P4-06`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `active`, `allowDeselect`, `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `autoComplete`, `checkIconPosition`, `children`, `clickOutsideCapture`, `color`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `disabled`, `dropdownOpened`, `dropdownPadding`, `error`, `filter`, `floatingHeight`, `floatingStrategy`, `form`, `hidden`, `hiddenInputProps`, `hiddenInputValuesDivider`, `hideDetached`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `label`, `labelledBy`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `middlewares`, `multiline`, `multiple`, `name`, `nothingFoundMessage`, `offset`, `onChange`, `onClear`, `onClose`, `onDismiss`, `onDropdownClose`, `onDropdownOpen`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onOptionSubmit`, `onPositionChange`, `onSearchChange`, `overlayProps`, `pointer`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `readOnly`, `refProp`, `renderOption`, `required`, `resetSelectionOnOptionHover`, `returnFocus`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnDropdownOpen`, `selected`, `shadow`, `size`, `store`, `success`, `targetType`, `transitionProps`, `value`, `valuesDivider`, `width`, `withAlignedLabels`, `withAria`, `withAriaAttributes`, `withArrow`, `withCheckIcon`, `withErrorStyles`, `withExpandedAttribute`, `withKeyboardNavigation`, `withOverlay`, `withScrollArea`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `zIndex`

**官方功能章节：** Examples；Usage；useCombobox hook；useCombobox options；Combobox store；useCombobox handlers；Combobox.Target；Split events and dropdown targets；Update selected option index；Search input；Select first option；Active option；Options groups；Scrollable list；Scrollable list with ScrollArea；Fit dropdown to viewport height；Hide dropdown；Control dropdown opened state；Popover props；Virtualization

**本地实现：** [src/combobox.rs](../src/combobox.rs)

**本地公开入口（同族）：** `class`, `clear_all`, `combobox`, `default_open`, `default_search`, `default_value`, `empty_message`, `enabled`, `field`, `filter`, `focus`, `form_name`, `group`, `invalid`, `label`, `name`, `node_ref`, `on`, `on_open_change`, `on_search_change`, `on_search_commit`, `on_value_change`, `open`, `option`, `options`, `placeholder`, `readonly`, `required`, `search`, `select_all`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `active`, `allowDeselect`, `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `autoComplete`, `checkIconPosition`, `children`, `clickOutsideCapture`, `color`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `disabled`, `dropdownOpened`, `dropdownPadding`, `error`, `floatingHeight`, `floatingStrategy`, `form`, `hidden`, `hiddenInputProps`, `hiddenInputValuesDivider`, `hideDetached`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `labelledBy`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `middlewares`, `multiline`, `multiple`, `nothingFoundMessage`, `offset`, `onChange`, `onClear`, `onClose`, `onDismiss`, `onDropdownClose`, `onDropdownOpen`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onOptionSubmit`, `onPositionChange`, `overlayProps`, `pointer`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `readOnly`, `refProp`, `renderOption`, `resetSelectionOnOptionHover`, `returnFocus`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnDropdownOpen`, `selected`, `shadow`, `store`, `success`, `targetType`, `transitionProps`, `valuesDivider`, `width`, `withAlignedLabels`, `withAria`, `withAriaAttributes`, `withArrow`, `withCheckIcon`, `withErrorStyles`, `withExpandedAttribute`, `withKeyboardNavigation`, `withOverlay`, `withScrollArea`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `zIndex`

### ComboboxPopover — M:ComboboxPopover

来源：[@mantine/core](https://mantine.dev/core/combobox-popover/)；对应 `P0-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `checkIconPosition`, `children`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `dropdownOpened`, `filter`, `form`, `hiddenInputProps`, `hiddenInputValuesDivider`, `limit`, `maxDropdownHeight`, `multiple`, `name`, `nothingFoundMessage`, `onChange`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `onSearchChange`, `renderOption`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnDropdownOpen`, `value`, `withAlignedLabels`, `withCheckIcon`, `withScrollArea`

**官方功能章节：** Usage；Data format；Data prop；Searchable；Controlled search value；Sort options；Limit options；Multiple selection；Check icon；Allow deselect；Nothing found message；Disabled options；Groups；Render option；Large data sets；Control dropdown opened state；Dropdown position；Dropdown width；Dropdown animation

**本地实现：** [../glory/crates/blitz/src/layers.rs](../../glory/crates/blitz/src/layers.rs)、[../glory/crates/web/src/browser/layer_document.rs](../../glory/crates/web/src/browser/layer_document.rs)、[src/overlay.rs](../src/overlay.rs)

**本地公开入口（同族）：** `allowed`, `ax_branch`, `boundary`, `cancel_label`, `class`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `enabled`, `initial_focus`, `label`, `modal`, `name`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_snap_change`, `open`, `panel_ref`, `placement`, `return_focus`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `allowDeselect`, `checkIconPosition`, `children`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `dropdownOpened`, `filter`, `form`, `hiddenInputProps`, `hiddenInputValuesDivider`, `limit`, `maxDropdownHeight`, `multiple`, `nothingFoundMessage`, `onChange`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `onSearchChange`, `renderOption`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnDropdownOpen`, `value`, `withAlignedLabels`, `withCheckIcon`, `withScrollArea`

### MultiSelect — M:MultiSelect

来源：[@mantine/core](https://mantine.dev/core/multi-select/)；对应 `P4-08`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `hiddenInputProps`, `hiddenInputValuesDivider`, `hidePickedOptions`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `maxValues`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onMaxValues`, `onOptionSubmit`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `renderPill`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `size`, `success`, `successProps`, `value`, `withAlignedLabels`, `withAsterisk`, `withCheckIcon`, `withErrorStyles`, `withPillsReorder`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Clearable；Searchable；Controlled search value；Nothing found；Checked option icon；Max selected values；Hide selected options；Data prop；Value type；Filtering；Sort options；Fuzzy search with fuse.js；Large datasets；renderOption；renderPill；Reorder pills；Scrollable dropdown；Fit dropdown to viewport height；Group options；Disabled options；Inside Popover；Control dropdown opened state；Dropdown position；Dropdown width；Dropdown offset；Dropdown animation；Dropdown padding；Dropdown shadow；Input sections；Input props；Read only；Disabled；Error state；Backspace key

**本地实现：** [src/combobox.rs](../src/combobox.rs)、[src/multi_select.rs](../src/multi_select.rs)、[src/tag.rs](../src/tag.rs)

**本地公开入口（同族）：** `class`, `clear_all`, `clear_label`, `clearable`, `combobox`, `default_open`, `default_search`, `default_value`, `empty_message`, `enabled`, `field`, `filter`, `focus`, `form_name`, `group`, `hide_picked_options`, `invalid`, `label`, `max_values`, `multi_select`, `name`, `node_ref`, `on`, `on_open_change`, `on_ready`, `on_search_change`, `on_search_commit`, `on_value_change`, `open`, `option`, `options`, `placeholder`, `readonly`, `required`, `search`, `select_all`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `floatingHeight`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onMaxValues`, `onOptionSubmit`, `onRemove`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `renderPill`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `success`, `successProps`, `withAlignedLabels`, `withAsterisk`, `withCheckIcon`, `withErrorStyles`, `withPillsReorder`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

### Pill — M:Pill

来源：[@mantine/core](https://mantine.dev/core/pill/)；对应 `P3-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `onRemove`, `pointer`, `radius`, `removeButtonProps`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `type`, `withAsterisk`, `withErrorStyles`, `withRemoveButton`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage

**本地实现：** [src/display.rs](../src/display.rs)、[src/tag.rs](../src/tag.rs)

**本地公开入口（同族）：** `actions`, `badge`, `bordered`, `card`, `class`, `content`, `default_visible`, `description`, `empty_state`, `enabled`, `fill`, `footer`, `header`, `heading_level`, `illustration`, `name`, `node_ref`, `on_remove`, `on_remove_press`, `paper`, `pill`, `removable`, `remove_ref`, `shadow`, `size`, `style`, `theme`, `title`, `unstyled`, `variant`, `visible`

**需要语义映射的属性候选：** `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `removeButtonProps`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `type`, `withAsterisk`, `withErrorStyles`, `withRemoveButton`, `withSuccessStyles`, `wrapperProps`

### PillsInput — M:PillsInput

来源：[@mantine/core](https://mantine.dev/core/pills-input/)；对应 `P4-09`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `type`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Input props；Searchable select example；Accessibility

**本地实现：** [src/pills_input.rs](../src/pills_input.rs)、[src/tag.rs](../src/tag.rs)、[src/tags_input.rs](../src/tags_input.rs)

**本地公开入口（同族）：** `class`, `default_search`, `default_value`, `duplicate_policy`, `enabled`, `field`, `focus`, `form_name`, `invalid`, `label`, `max_tags`, `name`, `node_ref`, `on`, `on_change`, `on_commit`, `on_reject`, `on_remove`, `pills_input`, `placeholder`, `readonly`, `required`, `separators`, `size`, `state`, `style`, `tags_input`, `theme`, `unstyled`

**需要语义映射的属性候选：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `pointer`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `type`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### Select — M:Select

来源：[@mantine/core](https://mantine.dev/core/select/)；对应 `P4-05`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `allowDeselect`, `autoSelectOnBlur`, `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `hiddenInputProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `size`, `success`, `successProps`, `value`, `withAlignedLabels`, `withAsterisk`, `withCheckIcon`, `withErrorStyles`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；onChange handler；autoSelectOnBlur；Clearable；Allow deselect；Open on focus；Searchable；Controlled search value；Nothing found；Checked option icon；Value type；Data prop；Filtering；Sort options；Fuzzy search with fuse.js；Large datasets；renderOption；Scrollable dropdown；Fit dropdown to viewport height；Group options；Disabled options；Inside Popover；Control dropdown opened state；Dropdown position；Dropdown width；Dropdown offset；Prevent horizontal infinite scrolling；Dropdown animation；Dropdown padding；Dropdown shadow；Input sections；Input props；Read only；Disabled；Error state

**本地实现：** [../glory/crates/render/src/choice.rs](../../glory/crates/render/src/choice.rs)、[src/select.rs](../src/select.rs)

**本地公开入口（同族）：** `choice`, `choice_remove_last`, `class`, `clearable`, `default_open`, `default_value`, `enabled`, `field`, `focus`, `form_name`, `group`, `invalid`, `label`, `name`, `node_ref`, `on`, `on_open_change`, `on_value_change`, `open`, `option`, `options`, `placeholder`, `readonly`, `required`, `resolve_active_descendant`, `select`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `allowDeselect`, `autoSelectOnBlur`, `checkIconPosition`, `chevronColor`, `clearButtonProps`, `clearSectionMode`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `hiddenInputProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onOptionSubmit`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `success`, `successProps`, `withAlignedLabels`, `withAsterisk`, `withCheckIcon`, `withErrorStyles`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

### TagsInput — M:TagsInput

来源：[@mantine/core](https://mantine.dev/core/tags-input/)；对应 `P4-09`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `acceptValueOnBlur`, `allowDuplicates`, `clearButtonProps`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `isDuplicate`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `maxTags`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onDuplicate`, `onMaxTags`, `onOptionSubmit`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `renderPill`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `size`, `splitChars`, `success`, `successProps`, `value`, `withAsterisk`, `withErrorStyles`, `withPillsReorder`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Loading state；Controlled；Controlled search value；Clearable；Max selected values；Accept value on blur；Allow duplicates；isDuplicate；Split chars；With suggestions；Data prop；Filtering；Sort options；Fuzzy search with fuse.js；Large datasets；renderOption；Pill customization；Reorder pills；Scrollable dropdown；Fit dropdown to viewport height；Group options；Disabled options；Inside Popover；Control dropdown opened state；Dropdown position；Dropdown animation；Dropdown width；Dropdown padding；Dropdown shadow；Input sections；Input props；Read only；Disabled；Error state

**本地实现：** [src/pills_input.rs](../src/pills_input.rs)、[src/tag.rs](../src/tag.rs)、[src/tags_input.rs](../src/tags_input.rs)

**本地公开入口（同族）：** `class`, `default_search`, `default_value`, `duplicate_policy`, `enabled`, `field`, `focus`, `form_name`, `invalid`, `label`, `max_tags`, `name`, `node_ref`, `on`, `on_change`, `on_commit`, `on_reject`, `on_remove`, `pills_input`, `placeholder`, `readonly`, `required`, `separators`, `size`, `state`, `style`, `tags_input`, `theme`, `unstyled`

**需要语义映射的属性候选：** `acceptValueOnBlur`, `allowDuplicates`, `clearButtonProps`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultSearchValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `filter`, `floatingHeight`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `isDuplicate`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `maxDropdownHeight`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onDuplicate`, `onMaxTags`, `onOptionSubmit`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderOption`, `renderPill`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `selectFirstOptionOnChange`, `selectFirstOptionOnDropdownOpen`, `splitChars`, `success`, `successProps`, `value`, `withAsterisk`, `withErrorStyles`, `withPillsReorder`, `withScrollArea`, `withSuccessStyles`, `wrapperProps`

### TreeSelect — M:TreeSelect

来源：[@mantine/core](https://mantine.dev/core/tree-select/)；对应 `P8-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `checkStrictly`, `checkedStrategy`, `chevronAriaLabels`, `chevronColor`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `clearable`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultExpandAll`, `defaultExpandedValues`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `expandOnClick`, `expandedValues`, `filter`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxDisplayedValues`, `maxDisplayedValuesContent`, `maxDropdownHeight`, `maxValues`, `mode`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onExpandedChange`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderNode`, `renderPill`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `size`, `success`, `successProps`, `value`, `withAsterisk`, `withErrorStyles`, `withLines`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Data prop；Selection modes；Controlled；Searchable；Nothing found；Clearable；Expand on click；Connecting lines；Check strictly；Checked strategy；Max values；renderNode；renderPill；Scrollable dropdown；Control dropdown opened state；Dropdown position；Dropdown width；Dropdown offset；Dropdown animation；Dropdown padding；Dropdown shadow；Keyboard navigation；Expand state；Input sections；Input props；Read only；Disabled；Error state；Success state；Accessibility

**本地实现：** [../glory/crates/render/src/choice.rs](../../glory/crates/render/src/choice.rs)、[../glory/crates/render/src/tree.rs](../../glory/crates/render/src/tree.rs)、[src/cascader.rs](../src/cascader.rs)、[src/tree.rs](../src/tree.rs)、[src/tree_choice_model.rs](../src/tree_choice_model.rs)、[src/tree_select.rs](../src/tree_select.rs)

**本地公开入口（同族）：** `as_str`, `cascader`, `checked`, `choice`, `choice_remove_last`, `class`, `clearable`, `columns`, `contains`, `default_expanded`, `default_open`, `default_selected`, `default_value`, `direction`, `enabled`, `expanded`, `field`, `file_icons`, `file_tree`, `focus`, `form_name`, `height`, `invalid`, `is_empty`, `key`, `keys`, `leaf`, `multiple`, `name`, `new`, `node_ref`, `nodes`, `on`, `on_activate`, `on_expanded_change`, `on_load_children`, `on_open_change`, `on_selection_change`, `on_selection_intent`, `on_value_change`, `open`, `path`, `paths`, `placeholder`, `policy`, `readonly`, `render_label`, `required`, `resolve_active_descendant`, `selected`, `selection_follows_focus`, `selection_mode`, `size`, `style`, `theme`, `toggle`, `tree`, `tree_select`, `typeahead`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allowDeselect`, `checkStrictly`, `checkedStrategy`, `chevronAriaLabels`, `chevronColor`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `comboboxProps`, `data`, `defaultDropdownOpened`, `defaultExpandAll`, `defaultExpandedValues`, `defaultSearchValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `expandOnClick`, `expandedValues`, `filter`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxDisplayedValues`, `maxDisplayedValuesContent`, `maxDropdownHeight`, `maxValues`, `mode`, `nothingFoundMessage`, `onChange`, `onClear`, `onDropdownClose`, `onDropdownOpen`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderNode`, `renderPill`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withLines`, `withSuccessStyles`, `wrapperProps`

### ActionIcon — M:ActionIcon

来源：[@mantine/core](https://mantine.dev/core/action-icon/)；对应 `P2-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `borderWidth`, `children`, `color`, `disabled`, `gradient`, `loaderProps`, `loading`, `orientation`, `radius`, `size`

**官方功能章节：** Usage；Size；Disabled state；Disabled state when ActionIcon is link；Customize disabled styles；Disabled button with Tooltip；Loading state；Loader props；Add custom variants；Customize variants colors；Add custom sizes；ActionIcon.Group；ActionIcon.GroupSection；Accessibility

**本地实现：** [src/actions.rs](../src/actions.rs)

**本地公开入口（同族）：** `action_icon`, `animated`, `burger`, `button`, `button_group`, `buttons`, `class`, `close_button`, `controls`, `default_open`, `direction`, `duration_millis`, `enabled`, `focus`, `icon`, `joined`, `labels`, `loop_focus`, `name`, `node_ref`, `on`, `on_open_change`, `on_press`, `open`, `orientation`, `size`, `style`, `theme`, `toolbar`, `unstyled`, `variant`

**需要语义映射的属性候选：** `autoContrast`, `borderWidth`, `children`, `color`, `disabled`, `gradient`, `loaderProps`, `loading`, `radius`

### Button — M:Button

来源：[@mantine/core](https://mantine.dev/core/button/)；对应 `P0-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `borderWidth`, `children`, `color`, `disabled`, `fullWidth`, `gradient`, `justify`, `leftSection`, `loaderProps`, `loading`, `orientation`, `radius`, `rightSection`, `size`

**官方功能章节：** Usage；Full width；Left and right sections；Sections position；Compact size；Disabled state；Disabled state when Button is link；Customize disabled styles；Disabled button with Tooltip；Loading state；Custom variants；Customize variant colors；Button.Group；Button.GroupSection

**本地实现：** [src/button.rs](../src/button.rs)、[src/theme/mod.rs](../src/theme/mod.rs)

**本地公开入口（同族）：** `blue_dark`, `blue_light`, `button`, `class`, `dark`, `enabled`, `fill`, `focus`, `from_default`, `green_dark`, `green_light`, `label`, `left_section`, `light`, `loading`, `name`, `new`, `node_ref`, `on`, `on_press`, `palette`, `right_section`, `size`, `spacing`, `style`, `stylesheet`, `theme`, `typography`, `unstyled`, `validate`, `variant`, `with_tokens`

**需要语义映射的属性候选：** `autoContrast`, `borderWidth`, `children`, `color`, `disabled`, `fullWidth`, `gradient`, `justify`, `loaderProps`, `orientation`, `radius`

### CloseButton — M:CloseButton

来源：[@mantine/core](https://mantine.dev/core/close-button/)；对应 `P2-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `disabled`, `icon`, `iconSize`, `radius`, `size`

**官方功能章节：** Usage；Change icon；Accessibility

**本地实现：** [src/actions.rs](../src/actions.rs)

**本地公开入口（同族）：** `action_icon`, `animated`, `burger`, `button`, `button_group`, `buttons`, `class`, `close_button`, `controls`, `default_open`, `direction`, `duration_millis`, `enabled`, `focus`, `icon`, `joined`, `labels`, `loop_focus`, `name`, `node_ref`, `on`, `on_open_change`, `on_press`, `open`, `orientation`, `size`, `style`, `theme`, `toolbar`, `unstyled`, `variant`

**需要语义映射的属性候选：** `children`, `disabled`, `iconSize`, `radius`

### CopyButton — M:CopyButton

来源：[@mantine/core](https://mantine.dev/core/copy-button/)；对应 `P10-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `timeout`, `value`

**官方功能章节：** Usage；Security；Timeout；Server components

**本地实现：** [../glory/crates/render/src/clipboard.rs](../../glory/crates/render/src/clipboard.rs)、[../glory/crates/web/src/browser/clipboard.rs](../../glory/crates/web/src/browser/clipboard.rs)、[src/copy_button.rs](../src/copy_button.rs)

**本地公开入口（同族）：** `admit`, `cancel`, `class`, `copied_label`, `copy_button`, `enabled`, `failed_label`, `feedback_delay`, `focus`, `label`, `matches`, `name`, `next`, `node_ref`, `observe`, `on`, `on_result`, `size`, `source`, `state_ref`, `style`, `theme`, `unstyled`, `validate`

**需要语义映射的属性候选：** `children`, `timeout`, `value`

### FileButton — M:FileButton

来源：[@mantine/core](https://mantine.dev/core/file-button/)；对应 `P10-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accept`, `capture`, `children`, `disabled`, `form`, `inputProps`, `multiple`, `name`, `onChange`, `resetRef`

**官方功能章节：** Usage；Multiple files；Reset file；Server components

**本地实现：** [../glory/crates/render/src/files.rs](../../glory/crates/render/src/files.rs)、[../glory/crates/web/src/browser/files.rs](../../glory/crates/web/src/browser/files.rs)、[src/files.rs](../src/files.rs)

**本地公开入口（同族）：** `attachment`, `class`, `clear`, `configuration`, `controller_ref`, `description`, `dropzone`, `enabled`, `file_attachment`, `file_button`, `file_input`, `files`, `files_ref`, `focus`, `label`, `matches`, `name`, `native_source`, `new`, `node_ref`, `on`, `on_files`, `on_result`, `preview`, `preview_in`, `previews`, `read`, `read_in`, `remove`, `size`, `status`, `status_ref`, `style`, `text`, `theme`, `unstyled`, `validate`, `validate_file`, `validate_files`, `validate_node`, `validate_read`, `validate_update`, `value`

**需要语义映射的属性候选：** `accept`, `capture`, `children`, `disabled`, `form`, `inputProps`, `multiple`, `onChange`, `resetRef`

### Toggle — M:Toggle

来源：[@mantine/core](https://mantine.dev/core/toggle/)；对应 `P2-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `autoContrast`, `autoWidth`, `color`, `defaultActive`, `disabled`, `onActiveChange`, `radius`, `size`

**官方功能章节：** Usage；Controlled and uncontrolled；Auto contrast；Auto width；Configurator；Accessibility

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/semantics.rs](../../glory/crates/html/src/semantics.rs)、[../glory/crates/render/src/semantics.rs](../../glory/crates/render/src/semantics.rs)、[src/toggle.rs](../src/toggle.rs)

**本地公开入口（同族）：** `as_str`, `class`, `default_pressed`, `default_value`, `enabled`, `focus`, `item`, `items`, `key`, `key_with_direction`, `label`, `loop_focus`, `mode`, `name`, `new`, `node_ref`, `on`, `on_pressed_change`, `on_value_change`, `orientation`, `pressed`, `size`, `style`, `theme`, `toggle`, `toggle_group`, `toggle_item`, `unstyled`, `value`

**需要语义映射的属性候选：** `active`, `autoContrast`, `autoWidth`, `color`, `defaultActive`, `disabled`, `onActiveChange`, `radius`

### UnstyledButton — M:UnstyledButton

来源：[@mantine/core](https://mantine.dev/core/unstyled-button/)；对应 `P0-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `size`

**官方功能章节：** Usage

**本地实现：** [src/button.rs](../src/button.rs)、[src/theme/mod.rs](../src/theme/mod.rs)

**本地公开入口（同族）：** `blue_dark`, `blue_light`, `button`, `class`, `dark`, `enabled`, `fill`, `focus`, `from_default`, `green_dark`, `green_light`, `label`, `left_section`, `light`, `loading`, `name`, `new`, `node_ref`, `on`, `on_press`, `palette`, `right_section`, `size`, `spacing`, `style`, `stylesheet`, `theme`, `typography`, `unstyled`, `validate`, `variant`, `with_tokens`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Anchor — M:Anchor

来源：[@mantine/core](https://mantine.dev/core/anchor/)；对应 `P3-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `gradient`, `inherit`, `inline`, `lineClamp`, `size`, `textWrap`, `truncate`, `underline`

**官方功能章节：** Usage；Underline；Text props

**本地实现：** [../glory/crates/render/tests/links.rs](../../glory/crates/render/tests/links.rs)、[src/anchor.rs](../src/anchor.rs)

**本地公开入口（同族）：** `anchor`, `class`, `current`, `focus`, `href`, `label`, `name`, `node_ref`, `on`, `on_navigate`, `open_in`, `size`, `style`, `theme`, `unstyled`

**需要语义映射的属性候选：** `gradient`, `inherit`, `inline`, `lineClamp`, `textWrap`, `truncate`, `underline`

### Breadcrumbs — M:Breadcrumbs

来源：[@mantine/core](https://mantine.dev/core/breadcrumbs/)；对应 `P6-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `separator`, `separatorMargin`

**官方功能章节：** Usage

**本地实现：** [../glory/crates/render/tests/current.rs](../../glory/crates/render/tests/current.rs)、[src/navigation.rs](../src/navigation.rs)

**本地公开入口（同族）：** `boundaries`, `breadcrumb_item`, `breadcrumbs`, `children`, `class`, `content_policy`, `current`, `default_expanded`, `default_page`, `enabled`, `expanded`, `has_next`, `highlighted`, `href`, `items`, `label`, `max_items`, `name`, `nav_link`, `node_ref`, `on`, `on_expanded_change`, `on_navigate`, `on_page_change`, `page`, `pagination`, `pagination_window`, `siblings`, `size`, `style`, `theme`, `total`, `unstyled`

**需要语义映射的属性候选：** `separator`, `separatorMargin`

### Burger — M:Burger

来源：[@mantine/core](https://mantine.dev/core/burger/)；对应 `P2-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `lineSize`, `opened`, `size`, `transitionDuration`, `transitionTimingFunction`

**官方功能章节：** Usage；Change line size；Accessibility

**本地实现：** [src/actions.rs](../src/actions.rs)

**本地公开入口（同族）：** `action_icon`, `animated`, `burger`, `button`, `button_group`, `buttons`, `class`, `close_button`, `controls`, `default_open`, `direction`, `duration_millis`, `enabled`, `focus`, `icon`, `joined`, `labels`, `loop_focus`, `name`, `node_ref`, `on`, `on_open_change`, `on_press`, `open`, `orientation`, `size`, `style`, `theme`, `toolbar`, `unstyled`, `variant`

**需要语义映射的属性候选：** `color`, `lineSize`, `opened`, `transitionDuration`, `transitionTimingFunction`

### NavLink — M:NavLink

来源：[@mantine/core](https://mantine.dev/core/nav-link/)；对应 `P6-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `autoContrast`, `children`, `childrenOffset`, `color`, `defaultOpened`, `description`, `disableRightSectionRotation`, `disabled`, `keepMounted`, `label`, `leftSection`, `noWrap`, `onChange`, `onClick`, `onKeyDown`, `opened`, `rightSection`

**官方功能章节：** Usage；Active；Nested NavLinks

**本地实现：** [src/navigation.rs](../src/navigation.rs)、[src/navigation_menu/mod.rs](../src/navigation_menu/mod.rs)

**本地公开入口（同族）：** `boundaries`, `breadcrumb_item`, `breadcrumbs`, `children`, `class`, `content`, `content_policy`, `current`, `default_expanded`, `default_open_path`, `default_page`, `direction`, `enabled`, `expanded`, `has_next`, `highlighted`, `hover`, `hover_delay_with`, `href`, `items`, `label`, `max_items`, `name`, `nav_link`, `navigation_menu`, `node_ref`, `on`, `on_expanded_change`, `on_navigate`, `on_open_path_change`, `on_page_change`, `open`, `open_path`, `page`, `pagination`, `pagination_window`, `siblings`, `size`, `style`, `theme`, `total`, `track_focusable`, `unstyled`

**需要语义映射的属性候选：** `active`, `autoContrast`, `childrenOffset`, `color`, `defaultOpened`, `description`, `disableRightSectionRotation`, `disabled`, `keepMounted`, `leftSection`, `noWrap`, `onChange`, `onClick`, `onKeyDown`, `opened`, `rightSection`

### Pagination — M:Pagination

来源：[@mantine/core](https://mantine.dev/core/pagination/)；对应 `P6-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `autoContrast`, `boundaries`, `color`, `defaultValue`, `disabled`, `dotsIcon`, `firstIcon`, `formatLabel`, `gap`, `getControlProps`, `getItemProps`, `hideWithOnePage`, `icon`, `lastIcon`, `layout`, `nextIcon`, `onChange`, `onFirstPage`, `onLastPage`, `onNextPage`, `onPreviousPage`, `previousIcon`, `radius`, `siblings`, `size`, `startValue`, `total`, `value`, `withControls`, `withEdges`, `withPadding`, `withPages`

**官方功能章节：** Usage；Example with chunked content；Controlled；Siblings；Boundaries；Responsive layout；Compound components；Controls as links；Change icons；Controls size；Start value；URL synchronization；use-pagination hook

**本地实现：** [../glory/crates/render/tests/current.rs](../../glory/crates/render/tests/current.rs)、[src/navigation.rs](../src/navigation.rs)

**本地公开入口（同族）：** `boundaries`, `breadcrumb_item`, `breadcrumbs`, `children`, `class`, `content_policy`, `current`, `default_expanded`, `default_page`, `enabled`, `expanded`, `has_next`, `highlighted`, `href`, `items`, `label`, `max_items`, `name`, `nav_link`, `node_ref`, `on`, `on_expanded_change`, `on_navigate`, `on_page_change`, `page`, `pagination`, `pagination_window`, `siblings`, `size`, `style`, `theme`, `total`, `unstyled`

**需要语义映射的属性候选：** `active`, `autoContrast`, `color`, `defaultValue`, `disabled`, `dotsIcon`, `firstIcon`, `formatLabel`, `gap`, `getControlProps`, `getItemProps`, `hideWithOnePage`, `icon`, `lastIcon`, `layout`, `nextIcon`, `onChange`, `onFirstPage`, `onLastPage`, `onNextPage`, `onPreviousPage`, `previousIcon`, `radius`, `startValue`, `value`, `withControls`, `withEdges`, `withPadding`, `withPages`

### Stepper — M:Stepper

来源：[@mantine/core](https://mantine.dev/core/stepper/)；对应 `P6-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `allowNextStepsSelect`, `allowStepClick`, `allowStepSelect`, `autoContrast`, `children`, `color`, `completedIcon`, `contentPadding`, `description`, `icon`, `iconPosition`, `iconSize`, `keepMounted`, `label`, `labelPosition`, `loading`, `onStepClick`, `orientation`, `progressIcon`, `radius`, `size`, `state`, `step`, `withIcon`, `wrap`

**官方功能章节：** Usage；Allow step select；Disable next steps selection；Color, radius and size；With custom icons；Vertical orientation；Label position；Icon position；Get step ref；Wrap Stepper.Step；Accessibility；Keyboard Navigation

**本地实现：** [src/stepper.rs](../src/stepper.rs)

**本地公开入口（同族）：** `active`, `allow_skip`, `class`, `completed`, `completed_label`, `content_policy`, `default_active`, `description`, `enabled`, `focus`, `name`, `node_ref`, `on`, `on_step_change`, `optional`, `orientation`, `selectable`, `size`, `status`, `step`, `stepper`, `steps`, `style`, `theme`, `unstyled`

**需要语义映射的属性候选：** `allowNextStepsSelect`, `allowStepClick`, `allowStepSelect`, `autoContrast`, `children`, `color`, `completedIcon`, `contentPadding`, `icon`, `iconPosition`, `iconSize`, `keepMounted`, `label`, `labelPosition`, `loading`, `onStepClick`, `progressIcon`, `radius`, `state`, `withIcon`, `wrap`

### TableOfContents — M:TableOfContents

来源：[@mantine/core](https://mantine.dev/core/table-of-contents/)；对应 `P6-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `color`, `depthOffset`, `getControlProps`, `initialData`, `minDepthToOffset`, `radius`, `reinitializeRef`, `scrollSpyOptions`, `size`

**官方功能章节：** Usage；use-scroll-spy options；Pass props to controls；Initial data；Depth offset；Styles API；Reinitialize

**本地实现：** [src/table_of_contents/mod.rs](../src/table_of_contents/mod.rs)、[src/table_of_contents/model.rs](../src/table_of_contents/model.rs)

**本地公开入口（同族）：** `class`, `content`, `current`, `entries`, `is_empty`, `len`, `name`, `node_ref`, `offset`, `order`, `refresh`, `reveal`, `size`, `style`, `table_of_contents`, `theme`, `toc_context`, `toc_document`, `toc_heading`, `unstyled`

**需要语义映射的属性候选：** `autoContrast`, `color`, `depthOffset`, `getControlProps`, `initialData`, `minDepthToOffset`, `radius`, `reinitializeRef`, `scrollSpyOptions`

### Tabs — M:Tabs

来源：[@mantine/core](https://mantine.dev/core/tabs/)；对应 `P0-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `activateTabWithKeyboard`, `allowTabDeactivation`, `autoContrast`, `children`, `color`, `defaultValue`, `grow`, `id`, `inverted`, `justify`, `keepMounted`, `keepMountedMode`, `leftSection`, `loop`, `onChange`, `orientation`, `placement`, `radius`, `rightSection`, `size`, `value`

**官方功能章节：** Usage；Controlled Tabs；Uncontrolled Tabs；Change colors；Tabs position；Inverted tabs；Vertical tabs placement；Custom variants；Disabled tabs；Activation mode；Tab deactivation；Unmount inactive tabs；Get tab control ref；Tab as a link；Tab as a link with Next.js；Tab as a link with React Router；Usage with react-router；Usage with Next.js Pages Router；Example with Scroller component；Accessibility；Keyboard interactions

**本地实现：** [src/tabs/dynamic.rs](../src/tabs/dynamic.rs)、[src/tabs/mod.rs](../src/tabs/mod.rs)

**本地公开入口（同族）：** `activation`, `class`, `content_policy`, `default_value`, `direction`, `enabled`, `focus`, `loop_focus`, `name`, `node_ref`, `on`, `on_value_change`, `orientation`, `size`, `style`, `tab`, `tabs`, `theme`, `trigger_content`, `unstyled`, `value`

**需要语义映射的属性候选：** `activateTabWithKeyboard`, `allowTabDeactivation`, `autoContrast`, `children`, `color`, `grow`, `id`, `inverted`, `justify`, `keepMounted`, `keepMountedMode`, `leftSection`, `loop`, `onChange`, `placement`, `radius`, `rightSection`

### Tree — M:Tree

来源：[@mantine/core](https://mantine.dev/core/tree/)；对应 `P8-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowDrop`, `allowRangeSelection`, `aspectRatio`, `autoContrast`, `checkOnSpace`, `checkStrictly`, `checkedStrategy`, `chevronAriaLabels`, `chevronColor`, `children`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `clearSelectionOnOutsideClick`, `clearable`, `comboboxProps`, `data`, `dataKey`, `defaultDropdownOpened`, `defaultExpandAll`, `defaultExpandedValues`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownOpened`, `error`, `errorProps`, `expandOnClick`, `expandOnSpace`, `expandedValues`, `filter`, `height`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `keepMounted`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `levelOffset`, `loading`, `loadingPosition`, `maxDisplayedValues`, `maxDisplayedValuesContent`, `maxDropdownHeight`, `maxValues`, `mode`, `nothingFoundMessage`, `onChange`, `onClear`, `onDragDrop`, `onDropdownClose`, `onDropdownOpen`, `onExpandedChange`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderNode`, `renderPill`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectOnClick`, `size`, `strokeColor`, `strokeWidth`, `success`, `successProps`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `tree`, `treemapProps`, `value`, `valueFormatter`, `withAsterisk`, `withDragHandle`, `withErrorStyles`, `withLines`, `withSuccessStyles`, `withTooltip`, `wrapperProps`

**官方功能章节：** Usage；Data prop；Data type；renderNode；useTree hook；Checked state；Check strictly；Initial expanded state；Async loading；Search and filter；Drag and drop；Connecting lines；Virtualization；Example: files tree；Example: docs navigation editor

**本地实现：** [../glory/crates/render/src/tree.rs](../../glory/crates/render/src/tree.rs)、[src/tree.rs](../src/tree.rs)、[src/tree_model.rs](../src/tree_model.rs)

**本地公开入口（同族）：** `as_str`, `checked`, `children`, `class`, `default_expanded`, `default_selected`, `direction`, `disabled`, `enabled`, `expanded`, `file_icons`, `file_tree`, `file_tree_nodes`, `focus`, `height`, `index`, `key`, `kind`, `message`, `name`, `new`, `node_ref`, `nodes`, `on`, `on_activate`, `on_expanded_change`, `on_load_children`, `on_selection_change`, `on_selection_intent`, `parent`, `render_label`, `selected`, `selection_follows_focus`, `selection_mode`, `size`, `style`, `theme`, `tree`, `typeahead`, `unstyled`, `valid_tree_key`, `validate`, `validate_tree_keys`, `visible`

**需要语义映射的属性候选：** `allowDeselect`, `allowDrop`, `allowRangeSelection`, `aspectRatio`, `autoContrast`, `checkOnSpace`, `checkStrictly`, `checkedStrategy`, `chevronAriaLabels`, `chevronColor`, `clearButtonProps`, `clearSearchOnChange`, `clearSectionMode`, `clearSelectionOnOutsideClick`, `clearable`, `comboboxProps`, `data`, `dataKey`, `defaultDropdownOpened`, `defaultExpandAll`, `defaultExpandedValues`, `defaultSearchValue`, `defaultValue`, `description`, `descriptionProps`, `dropdownOpened`, `error`, `errorProps`, `expandOnClick`, `expandOnSpace`, `expandedValues`, `filter`, `hiddenInputProps`, `hiddenInputValuesDivider`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `keepMounted`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `levelOffset`, `loading`, `loadingPosition`, `maxDisplayedValues`, `maxDisplayedValuesContent`, `maxDropdownHeight`, `maxValues`, `mode`, `nothingFoundMessage`, `onChange`, `onClear`, `onDragDrop`, `onDropdownClose`, `onDropdownOpen`, `onRemove`, `onSearchChange`, `openOnFocus`, `pointer`, `radius`, `renderNode`, `renderPill`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `searchValue`, `searchable`, `selectOnClick`, `strokeColor`, `strokeWidth`, `success`, `successProps`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `treemapProps`, `value`, `valueFormatter`, `withAsterisk`, `withDragHandle`, `withErrorStyles`, `withLines`, `withSuccessStyles`, `withTooltip`, `wrapperProps`

### Alert — M:Alert

来源：[@mantine/core](https://mantine.dev/core/alert/)；对应 `P5-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `closeButtonLabel`, `color`, `icon`, `onClose`, `radius`, `title`, `withCloseButton`

**官方功能章节：** Accessibility

**本地实现：** [src/alert.rs](../src/alert.rs)

**本地公开入口（同族）：** `alert`, `announcement`, `class`, `closable`, `close_label`, `content`, `fill`, `name`, `node_ref`, `on_close`, `severity`, `size`, `style`, `theme`, `title`, `unstyled`, `visible`

**需要语义映射的属性候选：** `autoContrast`, `closeButtonLabel`, `color`, `icon`, `radius`, `withCloseButton`

### EmptyState — M:EmptyState

来源：[@mantine/core](https://mantine.dev/core/empty-state/)；对应 `P3-17`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `align`, `children`, `color`, `description`, `icon`, `size`, `title`, `withIndicatorBackground`

**官方功能章节：** Usage；Compound components；Variant；Indicator background；Accessibility

**本地实现：** [src/display.rs](../src/display.rs)

**本地公开入口（同族）：** `actions`, `badge`, `bordered`, `card`, `class`, `content`, `default_visible`, `description`, `empty_state`, `enabled`, `fill`, `footer`, `header`, `heading_level`, `illustration`, `name`, `node_ref`, `on_remove`, `on_remove_press`, `paper`, `pill`, `removable`, `remove_ref`, `shadow`, `size`, `style`, `theme`, `title`, `unstyled`, `variant`, `visible`

**需要语义映射的属性候选：** `align`, `children`, `color`, `icon`, `withIndicatorBackground`

### Loader — M:Loader

来源：[@mantine/core](https://mantine.dev/core/loader/)；对应 `P5-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `color`, `loaders`, `size`, `type`

**官方功能章节：** Usage；Size prop；Adding custom loaders；children prop

**本地实现：** [../glory/crates/blitz/src/graphic_motion.rs](../../glory/crates/blitz/src/graphic_motion.rs)、[src/spinner.rs](../src/spinner.rs)

**本地公开入口（同族）：** `animated`, `class`, `decorative`, `diameter`, `finish_transitions`, `install`, `install_transition`, `loader`, `name`, `new`, `node_ref`, `period_millis`, `permissions`, `remove`, `set_motion_preference`, `size`, `spinner`, `style`, `theme`, `thickness`, `unstyled`, `visible`

**需要语义映射的属性候选：** `children`, `color`, `loaders`, `type`

### Notification — M:Notification

来源：[@mantine/core](https://mantine.dev/core/notification/)；对应 `P5-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDragDismiss`, `allowScrollDismiss`, `autoClose`, `children`, `closeButtonProps`, `color`, `containerWidth`, `icon`, `layout`, `limit`, `loaderProps`, `loading`, `notificationMaxHeight`, `onClose`, `pauseResetOnHover`, `portalProps`, `position`, `radius`, `renderNotification`, `store`, `title`, `transitionDuration`, `withAutoCloseProgress`, `withBorder`, `withCloseButton`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Notifications system features；Accessibility

**本地实现：** [src/notifications.rs](../src/notifications.rs)

**本地公开入口（同族）：** `action`, `announcement`, `capacity`, `class`, `clear`, `clock`, `closable`, `close_label`, `content`, `dismiss`, `name`, `new`, `node_ref`, `notification`, `notification_request`, `notifications`, `on_close`, `on_ready`, `pause`, `pause_on_hover`, `pending`, `severity`, `show`, `style`, `theme`, `timeout`, `title`, `toaster`, `unstyled`, `update`, `visible`, `visible_limit`

**需要语义映射的属性候选：** `allowDragDismiss`, `allowScrollDismiss`, `autoClose`, `children`, `closeButtonProps`, `color`, `containerWidth`, `icon`, `layout`, `limit`, `loaderProps`, `loading`, `notificationMaxHeight`, `pauseResetOnHover`, `portalProps`, `position`, `radius`, `renderNotification`, `store`, `transitionDuration`, `withAutoCloseProgress`, `withBorder`, `withCloseButton`, `withinPortal`, `zIndex`

### Progress — M:Progress

来源：[@mantine/core](https://mantine.dev/core/progress/)；对应 `P5-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `animated`, `autoContrast`, `color`, `orientation`, `radius`, `size`, `striped`, `transitionDuration`, `value`, `withAria`

**官方功能章节：** Usage；Compound components；Vertical orientation；With tooltips；Section width transition；Accessibility

**本地实现：** [../glory/crates/render/src/progress.rs](../../glory/crates/render/src/progress.rs)、[src/progress.rs](../src/progress.rs)

**本地公开入口（同族）：** `class`, `fraction`, `indeterminate`, `name`, `node_ref`, `progress`, `range`, `ring_progress`, `semi_circle_progress`, `size`, `style`, `theme`, `thickness`, `unstyled`, `validate`, `value_text`, `width`

**需要语义映射的属性候选：** `animated`, `autoContrast`, `color`, `orientation`, `radius`, `striped`, `transitionDuration`, `value`, `withAria`

### RingProgress — M:RingProgress

来源：[@mantine/core](https://mantine.dev/core/ring-progress/)；对应 `P5-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `label`, `rootColor`, `roundCaps`, `sectionGap`, `sections`, `size`, `startAngle`, `thickness`, `transitionDuration`

**官方功能章节：** Usage；Size, thickness & rounded caps；Sections tooltips；Section gaps；Start angle；Background color；Section events；Customize label；Filled segment transition

**本地实现：** [../glory/crates/render/src/progress.rs](../../glory/crates/render/src/progress.rs)、[src/progress.rs](../src/progress.rs)

**本地公开入口（同族）：** `class`, `fraction`, `indeterminate`, `name`, `node_ref`, `progress`, `range`, `ring_progress`, `semi_circle_progress`, `size`, `style`, `theme`, `thickness`, `unstyled`, `validate`, `value_text`, `width`

**需要语义映射的属性候选：** `label`, `rootColor`, `roundCaps`, `sectionGap`, `sections`, `startAngle`, `transitionDuration`

### SemiCircleProgress — M:SemiCircleProgress

来源：[@mantine/core](https://mantine.dev/core/semi-circle-progress/)；对应 `P5-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `emptySegmentColor`, `fillDirection`, `filledSegmentColor`, `label`, `labelPosition`, `orientation`, `size`, `thickness`, `transitionDuration`, `value`

**官方功能章节：** Usage；Change empty segment color；Change label position；Filled segment transition

**本地实现：** [../glory/crates/render/src/progress.rs](../../glory/crates/render/src/progress.rs)、[src/progress.rs](../src/progress.rs)

**本地公开入口（同族）：** `class`, `fraction`, `indeterminate`, `name`, `node_ref`, `progress`, `range`, `ring_progress`, `semi_circle_progress`, `size`, `style`, `theme`, `thickness`, `unstyled`, `validate`, `value_text`, `width`

**需要语义映射的属性候选：** `emptySegmentColor`, `fillDirection`, `filledSegmentColor`, `label`, `labelPosition`, `orientation`, `transitionDuration`, `value`

### Skeleton — M:Skeleton

来源：[@mantine/core](https://mantine.dev/core/skeleton/)；对应 `P5-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `animate`, `circle`, `height`, `radius`, `visible`, `width`

**官方功能章节：** Usage；With content

**本地实现：** [src/skeleton.rs](../src/skeleton.rs)

**本地公开入口（同族）：** `circle`, `class`, `height`, `loading`, `name`, `node_ref`, `radius`, `skeleton`, `style`, `theme`, `unstyled`, `width`

**需要语义映射的属性候选：** `animate`, `visible`

### ActionBar — M:ActionBar

来源：[@mantine/core](https://mantine.dev/core/action-bar/)；对应 `P6-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `aria-label`, `closeOnEscape`, `keepMounted`, `onClose`, `opened`, `portalProps`, `position`, `radius`, `shadow`, `transitionProps`, `withBorder`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；With ActionIcon buttons；Placement；Close on Escape；Transitions；keepMounted

**本地实现：** [src/toolbar.rs](../src/toolbar.rs)

**本地公开入口（同族）：** `action_bar`, `button`, `buttons`, `class`, `clear_label`, `direction`, `enabled`, `group`, `groups`, `loop_focus`, `name`, `node_ref`, `on`, `on_clear_selection`, `orientation`, `selected_count`, `size`, `style`, `theme`, `toolbar`, `toolbar_group`, `unstyled`, `visible`

**需要语义映射的属性候选：** `aria-label`, `closeOnEscape`, `keepMounted`, `onClose`, `opened`, `portalProps`, `position`, `radius`, `shadow`, `transitionProps`, `withBorder`, `withinPortal`, `zIndex`

### Affix — M:Affix

来源：[@mantine/core](https://mantine.dev/core/affix/)；对应 `P6-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `portalProps`, `position`, `withinPortal`, `zIndex`

**官方功能章节：** Usage

**本地实现：** [src/affix.rs](../src/affix.rs)

**本地公开入口（同族）：** `affix`, `affix_container`, `class`, `content`, `fill`, `focus`, `name`, `node`, `node_ref`, `offset`, `on`, `size`, `sticky`, `style`, `theme`, `unstyled`, `validate`, `visible`, `within`

**需要语义映射的属性候选：** `portalProps`, `position`, `withinPortal`, `zIndex`

### Dialog — M:Dialog

来源：[@mantine/core](https://mantine.dev/core/dialog/)；对应 `P5-12`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `keepMounted`, `onClose`, `opened`, `portalProps`, `position`, `radius`, `shadow`, `size`, `transitionProps`, `withBorder`, `withCloseButton`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Change position；Accessibility

**本地实现：** [src/overlay.rs](../src/overlay.rs)

**本地公开入口（同族）：** `cancel_label`, `class`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `enabled`, `initial_focus`, `label`, `modal`, `name`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_snap_change`, `open`, `panel_ref`, `placement`, `return_focus`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `children`, `keepMounted`, `onClose`, `opened`, `portalProps`, `position`, `radius`, `shadow`, `transitionProps`, `withBorder`, `withCloseButton`, `withinPortal`, `zIndex`

### Drawer — M:Drawer

来源：[@mantine/core](https://mantine.dev/core/drawer/)；对应 `P0-08`；原生有限能力状态 `rejected`。

**官方 Props / 子部件 Props：** `children`, `closeButtonProps`, `closeOnClickOutside`, `closeOnEscape`, `id`, `keepMounted`, `keepMountedMode`, `lockScroll`, `offset`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `opened`, `overlayProps`, `padding`, `portalProps`, `position`, `radius`, `removeScrollProps`, `returnFocus`, `scrollAreaComponent`, `shadow`, `size`, `stackId`, `title`, `transitionProps`, `trapFocus`, `withCloseButton`, `withOverlay`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Position；Offset；Customize overlay；Sizes；Remove header；Drawer with scroll；Usage with ScrollArea；Change transition；onExitTransitionEnd and onEnterTransitionEnd；Initial focus；Control behavior；react-remove-scroll settings；Change close icon；Compound components；Drawer.Stack；useDrawersStack hook；Fixed elements offset；Accessibility

**本地实现：** [src/overlay.rs](../src/overlay.rs)、[src/sheet.rs](../src/sheet.rs)

**本地公开入口（同族）：** `cancel_label`, `class`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `dismissible`, `enabled`, `initial_focus`, `label`, `max_extent`, `modal`, `name`, `new`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_snap_change`, `open`, `panel_ref`, `placement`, `return_focus`, `settle_millis`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`, `validate`

**需要语义映射的属性候选：** `children`, `closeButtonProps`, `closeOnClickOutside`, `closeOnEscape`, `id`, `keepMounted`, `keepMountedMode`, `lockScroll`, `offset`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `opened`, `overlayProps`, `padding`, `portalProps`, `position`, `radius`, `removeScrollProps`, `scrollAreaComponent`, `shadow`, `stackId`, `transitionProps`, `trapFocus`, `withCloseButton`, `withOverlay`, `withinPortal`, `zIndex`

### FloatingIndicator — M:FloatingIndicator

来源：[@mantine/core](https://mantine.dev/core/floating-indicator/)；对应 `P6-13`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `displayAfterTransitionEnd`, `onTransitionEnd`, `onTransitionStart`, `parent`, `target`, `transitionDuration`

**官方功能章节：** Usage；Multiple rows；Example: Tabs

**本地实现：** [src/floating_indicator/mod.rs](../src/floating_indicator/mod.rs)、[src/floating_indicator/model.rs](../src/floating_indicator/model.rs)

**本地公开入口（同族）：** `active`, `animated`, `class`, `color`, `content`, `context_ref`, `duration`, `fill`, `floating_indicator`, `floating_indicator_target`, `geometry`, `measurement_revision`, `node_ref`, `on`, `on_geometry_change`, `pending_queries`, `rectangle`, `refresh`, `registered`, `style`, `theme`, `thickness`, `unstyled`, `visible`

**需要语义映射的属性候选：** `displayAfterTransitionEnd`, `onTransitionEnd`, `onTransitionStart`, `parent`, `target`, `transitionDuration`

### FloatingWindow — M:FloatingWindow

来源：[@mantine/core](https://mantine.dev/core/floating-window/)；对应 `P5-14`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `axis`, `constrainOffset`, `constrainToViewport`, `dimensions`, `dragHandleSelector`, `enabled`, `excludeDragHandleSelector`, `initialPosition`, `onDragEnd`, `onDragStart`, `onPositionChange`, `onResizeEnd`, `onResizeStart`, `onSizeChange`, `portalProps`, `radius`, `setPositionRef`, `shadow`, `withBorder`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Constrain to viewport；Constrain offset；Drag handle selector；Enabled prop；Set position；Resize handle；Resize callbacks；Lock axis；use-floating-window hook

**本地实现：** [../glory/crates/render/src/viewport_layer.rs](../../glory/crates/render/src/viewport_layer.rs)、[src/floating_window/mod.rs](../src/floating_window/mod.rs)

**本地公开入口（同族）：** `bounds`, `class`, `close`, `command`, `constrain_to_viewport`, `content`, `content_policy`, `default_bounds`, `default_open`, `enabled`, `floating_window`, `focus`, `keyboard_step`, `limits`, `name`, `node_ref`, `on`, `on_bounds_change`, `on_dismiss`, `on_open_change`, `on_ready`, `open`, `raise`, `resolve`, `set_bounds`, `size`, `style`, `theme`, `title`, `unstyled`, `validate`, `viewport_padding`

**需要语义映射的属性候选：** `axis`, `constrainOffset`, `dimensions`, `dragHandleSelector`, `excludeDragHandleSelector`, `initialPosition`, `onDragEnd`, `onDragStart`, `onPositionChange`, `onResizeEnd`, `onResizeStart`, `onSizeChange`, `portalProps`, `radius`, `setPositionRef`, `shadow`, `withBorder`, `withinPortal`, `zIndex`

### HoverCard — M:HoverCard

来源：[@mantine/core](https://mantine.dev/core/hover-card/)；对应 `P5-04`；原生有限能力状态 `rejected`。

**官方 Props / 子部件 Props：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `children`, `clickOutsideCapture`, `clickOutsideEvents`, `closeDelay`, `closeOnClickOutside`, `closeOnEscape`, `defaultOpened`, `disabled`, `eventPropsWrapperName`, `events`, `floatingStrategy`, `hideDetached`, `id`, `initiallyOpened`, `interactive`, `keepMounted`, `keepMountedMode`, `middlewares`, `offset`, `onClose`, `onDismiss`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onPositionChange`, `openDelay`, `overlayProps`, `popupType`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `returnFocus`, `role`, `shadow`, `transitionProps`, `trapFocus`, `width`, `withArrow`, `withOverlay`, `withRoles`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Delays；HoverCard delay group；Activation events；Keyboard interactions；Interactive dropdown；With interactive elements；Dropdown role；Target component；Accessibility

**本地实现：** [src/hint.rs](../src/hint.rs)、[src/state.rs](../src/state.rs)

**本地公开入口（同族）：** `accept`, `cancel`, `class`, `close_delay_with`, `content`, `content_policy`, `default_open`, `enabled`, `hover_card`, `id`, `label`, `mount`, `name`, `new`, `node_ref`, `on`, `on_dismiss`, `on_open_change`, `open`, `open_delay_with`, `placement`, `run`, `size`, `style`, `take`, `target`, `text`, `theme`, `tooltip`, `touch`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `children`, `clickOutsideCapture`, `clickOutsideEvents`, `closeDelay`, `closeOnClickOutside`, `closeOnEscape`, `defaultOpened`, `disabled`, `eventPropsWrapperName`, `events`, `floatingStrategy`, `hideDetached`, `initiallyOpened`, `interactive`, `keepMounted`, `keepMountedMode`, `middlewares`, `offset`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onPositionChange`, `openDelay`, `overlayProps`, `popupType`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `returnFocus`, `role`, `shadow`, `transitionProps`, `trapFocus`, `width`, `withArrow`, `withOverlay`, `withRoles`, `withinPortal`, `zIndex`

### LoadingOverlay — M:LoadingOverlay

来源：[@mantine/core](https://mantine.dev/core/loading-overlay/)；对应 `P5-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `loaderProps`, `onEnter`, `onEntered`, `onExit`, `onExited`, `overlayProps`, `transitionProps`, `visible`, `zIndex`

**官方功能章节：** Usage；Loader props；Custom inline loaders

**本地实现：** [src/loading_overlay.rs](../src/loading_overlay.rs)

**本地公开入口（同族）：** `class`, `color`, `content_ref`, `fill`, `label`, `loading`, `loading_overlay`, `mask_ref`, `name`, `node_ref`, `opacity`, `overlay`, `style`, `theme`, `unstyled`, `visible`

**需要语义映射的属性候选：** `loaderProps`, `onEnter`, `onEntered`, `onExit`, `onExited`, `overlayProps`, `transitionProps`, `zIndex`

### Menu — M:Menu

来源：[@mantine/core](https://mantine.dev/core/menu/)；对应 `P4-01`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `alignItemsLabels`, `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `checkIcon`, `checked`, `children`, `clearSearchOnClose`, `clickOutsideCapture`, `clickOutsideEvents`, `closeDelay`, `closeMenuOnClick`, `closeOnClickOutside`, `closeOnEscape`, `closeOnItemClick`, `color`, `defaultChecked`, `defaultOpenIndex`, `defaultOpened`, `defaultValue`, `disabled`, `error`, `floatingStrategy`, `hideDetached`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `longPressDelay`, `loop`, `menuItemTabIndex`, `middlewares`, `multiline`, `offset`, `onChange`, `onClose`, `onDismiss`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onOpenChange`, `onPositionChange`, `openDelay`, `openIndex`, `opened`, `overlayProps`, `pointer`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `required`, `returnFocus`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `safeAreaPolygon`, `shadow`, `size`, `success`, `transitionProps`, `trapFocus`, `trigger`, `value`, `width`, `withAria`, `withArrow`, `withErrorStyles`, `withInitialFocusPlaceholder`, `withOverlay`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `zIndex`

**官方功能章节：** Usage；Submenus；Search；Checkbox items；Checkbox group；Radio items；Aligning labels of items with and without indicators；Custom check icon；Type-ahead navigation；Context menu；Controlled；Show menu on hover；Disabled items；Dropdown position；Transitions；Custom component as Menu.Item；Accessibility

**本地实现：** [src/menu/mod.rs](../src/menu/mod.rs)

**本地公开入口（同族）：** `anchor`, `checked`, `class`, `close_on_select`, `context_menu`, `default_checked`, `default_open`, `default_value`, `direction`, `dropdown_menu`, `editor_policy`, `enabled`, `focus`, `hover_delay_with`, `item`, `items`, `label`, `loop_focus`, `menu`, `menu_checkbox`, `menu_item`, `menu_radio_group`, `menu_radio_item`, `menu_submenu`, `name`, `node_ref`, `on`, `on_activate`, `on_checked_change`, `on_open_change`, `on_value_change`, `open`, `radio_group`, `return_focus`, `separator`, `size`, `style`, `submenu`, `target`, `target_name`, `target_ref`, `theme`, `trigger_ref`, `typeahead`, `unstyled`, `value`

**需要语义映射的属性候选：** `alignItemsLabels`, `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `checkIcon`, `children`, `clearSearchOnClose`, `clickOutsideCapture`, `clickOutsideEvents`, `closeDelay`, `closeMenuOnClick`, `closeOnClickOutside`, `closeOnEscape`, `closeOnItemClick`, `color`, `defaultOpenIndex`, `defaultOpened`, `disabled`, `error`, `floatingStrategy`, `hideDetached`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `longPressDelay`, `loop`, `menuItemTabIndex`, `middlewares`, `multiline`, `offset`, `onChange`, `onClose`, `onDismiss`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onPositionChange`, `openDelay`, `openIndex`, `opened`, `overlayProps`, `pointer`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `safeAreaPolygon`, `shadow`, `success`, `transitionProps`, `trapFocus`, `trigger`, `width`, `withAria`, `withArrow`, `withErrorStyles`, `withInitialFocusPlaceholder`, `withOverlay`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `zIndex`

### Menubar — M:Menubar

来源：[@mantine/core](https://mantine.dev/core/menubar/)；对应 `P4-03`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `children`, `defaultOpenIndex`, `loop`, `onOpenChange`, `openIndex`, `position`, `trigger`

**官方功能章节：** Usage；How it differs from Menu；Composition；Trigger and loop；Submenus；Checkbox and radio items；Controlled；Position；Accessibility

**本地实现：** [../glory/crates/render/src/semantics.rs](../../glory/crates/render/src/semantics.rs)、[src/menu/bar/mod.rs](../src/menu/bar/mod.rs)、[src/menu/mod.rs](../src/menu/mod.rs)

**本地公开入口（同族）：** `anchor`, `as_str`, `checked`, `class`, `close_on_select`, `context_menu`, `default_checked`, `default_open`, `default_value`, `direction`, `dropdown_menu`, `editor_policy`, `enabled`, `focus`, `hover_delay_with`, `item`, `items`, `key`, `key_with_direction`, `label`, `loop_focus`, `menu`, `menu_checkbox`, `menu_item`, `menu_radio_group`, `menu_radio_item`, `menu_submenu`, `menubar`, `menubar_menu`, `menus`, `name`, `new`, `node_ref`, `on`, `on_activate`, `on_checked_change`, `on_open_change`, `on_value_change`, `open`, `radio_group`, `return_focus`, `separator`, `size`, `style`, `submenu`, `target`, `target_name`, `target_ref`, `theme`, `trigger_ref`, `typeahead`, `unstyled`, `value`

**需要语义映射的属性候选：** `children`, `defaultOpenIndex`, `loop`, `openIndex`, `position`, `trigger`

### Modal — M:Modal

来源：[@mantine/core](https://mantine.dev/core/modal/)；对应 `P0-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `centered`, `children`, `closeButtonProps`, `closeOnClickOutside`, `closeOnEscape`, `fullScreen`, `id`, `keepMounted`, `keepMountedMode`, `labels`, `lockScroll`, `modalProps`, `modals`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `opened`, `overlayProps`, `padding`, `portalProps`, `radius`, `removeScrollProps`, `returnFocus`, `scrollAreaComponent`, `shadow`, `size`, `stackId`, `title`, `transitionProps`, `trapFocus`, `withCloseButton`, `withOverlay`, `withinPortal`, `xOffset`, `yOffset`, `zIndex`

**官方功能章节：** Usage；Center modal vertically；Remove header；Change size；Size auto；Fullscreen；Customize overlay；Modal with scroll；Usage with ScrollArea；Change offsets；Change transitions；onExitTransitionEnd and onEnterTransitionEnd；Initial focus；Control behavior；react-remove-scroll settings；Change close icon；Compound components；Modal.Stack；useModalsStack hook；Fixed elements offset；Accessibility

**本地实现：** [../glory/crates/blitz/src/layers.rs](../../glory/crates/blitz/src/layers.rs)、[../glory/crates/web/src/browser/layer_document.rs](../../glory/crates/web/src/browser/layer_document.rs)、[src/overlay.rs](../src/overlay.rs)

**本地公开入口（同族）：** `allowed`, `ax_branch`, `boundary`, `cancel_label`, `class`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `enabled`, `initial_focus`, `label`, `modal`, `name`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_snap_change`, `open`, `panel_ref`, `placement`, `return_focus`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `centered`, `children`, `closeButtonProps`, `closeOnClickOutside`, `closeOnEscape`, `fullScreen`, `id`, `keepMounted`, `keepMountedMode`, `labels`, `lockScroll`, `modalProps`, `modals`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `opened`, `overlayProps`, `padding`, `portalProps`, `radius`, `removeScrollProps`, `scrollAreaComponent`, `shadow`, `stackId`, `transitionProps`, `trapFocus`, `withCloseButton`, `withOverlay`, `withinPortal`, `xOffset`, `yOffset`, `zIndex`

### Overlay — M:Overlay

来源：[@mantine/core](https://mantine.dev/core/overlay/)；对应 `P5-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `backgroundOpacity`, `blur`, `center`, `children`, `color`, `fixed`, `gradient`, `radius`, `zIndex`

**官方功能章节：** Usage；Gradient；Blur

**本地实现：** [src/loading_overlay.rs](../src/loading_overlay.rs)

**本地公开入口（同族）：** `class`, `color`, `content_ref`, `fill`, `label`, `loading`, `loading_overlay`, `mask_ref`, `name`, `node_ref`, `opacity`, `overlay`, `style`, `theme`, `unstyled`, `visible`

**需要语义映射的属性候选：** `backgroundOpacity`, `blur`, `center`, `children`, `fixed`, `gradient`, `radius`, `zIndex`

### Popover — M:Popover

来源：[@mantine/core](https://mantine.dev/core/popover/)；对应 `P0-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `children`, `clickOutsideCapture`, `clickOutsideEvents`, `closeOnClickOutside`, `closeOnEscape`, `defaultOpened`, `disabled`, `floatingStrategy`, `hideDetached`, `id`, `keepMounted`, `keepMountedMode`, `longPressDelay`, `middlewares`, `offset`, `onChange`, `onClose`, `onDismiss`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onPositionChange`, `opened`, `overlayProps`, `popupType`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `returnFocus`, `shadow`, `transitionProps`, `trapFocus`, `width`, `withArrow`, `withOverlay`, `withRoles`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Controlled；Focus trap；Inline elements；Same width；offset；Middlewares；Customize middleware options；Dropdown arrow；With overlay；Hide detached；Disabled；Click outside；onDismiss；Initial focus；Target component；Context menu；Nested popovers；Accessibility；Keyboard interactions

**本地实现：** [../glory/crates/blitz/src/layers.rs](../../glory/crates/blitz/src/layers.rs)、[../glory/crates/web/src/browser/layer_document.rs](../../glory/crates/web/src/browser/layer_document.rs)、[src/overlay.rs](../src/overlay.rs)

**本地公开入口（同族）：** `allowed`, `ax_branch`, `boundary`, `cancel_label`, `class`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `enabled`, `initial_focus`, `label`, `modal`, `name`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_snap_change`, `open`, `panel_ref`, `placement`, `return_focus`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `children`, `clickOutsideCapture`, `clickOutsideEvents`, `closeOnClickOutside`, `closeOnEscape`, `defaultOpened`, `disabled`, `floatingStrategy`, `hideDetached`, `id`, `keepMounted`, `keepMountedMode`, `longPressDelay`, `middlewares`, `offset`, `onChange`, `onClose`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onOpen`, `onPositionChange`, `opened`, `overlayProps`, `popupType`, `portalProps`, `position`, `preventPositionChangeWhenVisible`, `radius`, `refProp`, `shadow`, `transitionProps`, `trapFocus`, `width`, `withArrow`, `withOverlay`, `withRoles`, `withinPortal`, `zIndex`

### Tooltip — M:Tooltip

来源：[@mantine/core](https://mantine.dev/core/tooltip/)；对应 `P5-03`；原生有限能力状态 `rejected`。

**官方 Props / 子部件 Props：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `autoContrast`, `children`, `closeDelay`, `color`, `defaultOpened`, `disabled`, `events`, `floatingStrategy`, `hideDetached`, `inline`, `interactive`, `keepMounted`, `label`, `middlewares`, `multiline`, `offset`, `onPositionChange`, `openDelay`, `opened`, `portalProps`, `position`, `radius`, `refProp`, `target`, `transitionProps`, `withArrow`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Tooltip children；Tooltip target；Hide detached；Required ref prop；Color；Offset；Arrow；Controlled；Change events；Interactive tooltip；Multiline；Inline；Change transition；Close and open delay；Tooltip delay group；Floating tooltip；Accessibility

**本地实现：** [src/hint.rs](../src/hint.rs)、[src/state.rs](../src/state.rs)

**本地公开入口（同族）：** `accept`, `cancel`, `class`, `close_delay_with`, `content`, `content_policy`, `default_open`, `enabled`, `hover_card`, `id`, `label`, `mount`, `name`, `new`, `node_ref`, `on`, `on_dismiss`, `on_open_change`, `open`, `open_delay_with`, `placement`, `run`, `size`, `style`, `take`, `target`, `text`, `theme`, `tooltip`, `touch`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `arrowOffset`, `arrowPosition`, `arrowRadius`, `arrowSize`, `autoContrast`, `children`, `closeDelay`, `color`, `defaultOpened`, `disabled`, `events`, `floatingStrategy`, `hideDetached`, `inline`, `interactive`, `keepMounted`, `middlewares`, `multiline`, `offset`, `onPositionChange`, `openDelay`, `opened`, `portalProps`, `position`, `radius`, `refProp`, `transitionProps`, `withArrow`, `withinPortal`, `zIndex`

### Tour — M:Tour

来源：[@mantine/core](https://mantine.dev/core/tour/)；对应 `P5-15`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `children`, `closeOnEscape`, `closeOnOverlayClick`, `color`, `defaultStep`, `labels`, `maxWidth`, `mode`, `mounted`, `onClick`, `onClose`, `onStepChange`, `onStepClose`, `onStepOpen`, `overlayColor`, `portalProps`, `position`, `scrollToHandler`, `spotlightPadding`, `spotlightRadius`, `step`, `stepTransitionDuration`, `stepsCount`, `target`, `targetElement`, `targetRect`, `title`, `tooltipShadow`, `transitionProps`, `withBeacon`, `withCloseButton`, `withInteraction`, `withKeyboardNavigation`, `withOverlay`, `withOverlayInteraction`, `withScrollIntoView`, `withinPortal`, `zIndex`

**官方功能章节：** Usage；Complex example；Beacon mode；Controlled；Overlay configuration；Step targets；Steps without target；Custom content；Tooltip position；Custom labels；Styles API；Keyboard navigation；Accessibility

**本地实现：** [../glory/crates/blitz/src/scroll_into_view.rs](../../glory/crates/blitz/src/scroll_into_view.rs)、[src/tour.rs](../src/tour.rs)

**本地公开入口（同族）：** `cancel`, `current`, `default_current`, `default_open`, `fill`, `go_to`, `is_empty`, `len`, `next`, `on_cancel`, `on_complete`, `on_current_change`, `on_open_change`, `on_ready`, `on_target_lost`, `open`, `placement`, `previous`, `refresh`, `step`, `steps`, `theme`, `tour`, `tour_context`, `tour_step`, `tour_target`, `unstyled`

**需要语义映射的属性候选：** `active`, `children`, `closeOnEscape`, `closeOnOverlayClick`, `color`, `defaultStep`, `labels`, `maxWidth`, `mode`, `mounted`, `onClick`, `onClose`, `onStepChange`, `onStepClose`, `onStepOpen`, `overlayColor`, `portalProps`, `position`, `scrollToHandler`, `spotlightPadding`, `spotlightRadius`, `stepTransitionDuration`, `stepsCount`, `target`, `targetElement`, `targetRect`, `title`, `tooltipShadow`, `transitionProps`, `withBeacon`, `withCloseButton`, `withInteraction`, `withKeyboardNavigation`, `withOverlay`, `withOverlayInteraction`, `withScrollIntoView`, `withinPortal`, `zIndex`

### Accordion — M:Accordion

来源：[@mantine/core](https://mantine.dev/core/accordion/)；对应 `P0-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `chevron`, `chevronIconSize`, `chevronPosition`, `chevronSize`, `children`, `defaultValue`, `disableChevronRotation`, `disableCollapse`, `disabled`, `icon`, `keepMounted`, `keepMountedMode`, `loop`, `multiple`, `onChange`, `order`, `radius`, `transitionDuration`, `value`

**官方功能章节：** Usage；order prop；Change chevron；Custom control label；With icons；Change transition；Default opened items；Control opened state；Disable collapse；Compose controls；Disabled items；TypeScript；Accessibility；Keyboard interactions

**本地实现：** [src/accordion/mod.rs](../src/accordion/mod.rs)、[src/collection.rs](../src/collection.rs)、[src/composite/mod.rs](../src/composite/mod.rs)

**本地公开入口（同族）：** `accept`, `accordion`, `accordion_item`, `class`, `clear`, `collapsible`, `content`, `content_policy`, `current`, `default_open`, `destination`, `direction`, `emit`, `enabled`, `flags`, `focus`, `heading_level`, `is_tab_stop`, `item`, `items`, `label`, `locate`, `loop_focus`, `mounted`, `multiple`, `name`, `navigate`, `new`, `node_ref`, `observe`, `on`, `on_open_change`, `open`, `panel_ref`, `query`, `regions`, `register`, `register_ordered`, `registered_len`, `repair`, `search`, `size`, `style`, `theme`, `trigger_content`, `trigger_ref`, `unstyled`

**需要语义映射的属性候选：** `chevron`, `chevronIconSize`, `chevronPosition`, `chevronSize`, `children`, `defaultValue`, `disableChevronRotation`, `disableCollapse`, `disabled`, `icon`, `keepMounted`, `keepMountedMode`, `loop`, `onChange`, `order`, `radius`, `transitionDuration`, `value`

### Avatar — M:Avatar

来源：[@mantine/core](https://mantine.dev/core/avatar/)；对应 `P3-13`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowedInitialsColors`, `alt`, `autoContrast`, `children`, `color`, `gradient`, `imageProps`, `name`, `radius`, `size`, `src`

**官方功能章节：** Usage；Initials；Allowed initials colors；Placeholder；Variants；Avatar.Group；Accessibility

**本地实现：** [src/avatar.rs](../src/avatar.rs)、[src/image.rs](../src/image.rs)

**本地公开入口（同族）：** `alt`, `avatar`, `avatar_group`, `avatars`, `background_image`, `class`, `decorative`, `fallback`, `fallback_src`, `fill`, `fit`, `focus`, `height`, `image`, `max_visible`, `name`, `node_ref`, `on`, `on_error`, `on_load`, `overflow_label`, `overlap`, `shape`, `size`, `source`, `style`, `theme`, `unstyled`, `width`

**需要语义映射的属性候选：** `allowedInitialsColors`, `autoContrast`, `children`, `color`, `gradient`, `imageProps`, `radius`, `src`

### BackgroundImage — M:BackgroundImage

来源：[@mantine/core](https://mantine.dev/core/background-image/)；对应 `P3-14`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `radius`, `src`

**官方功能章节：** Usage

**本地实现：** [src/image.rs](../src/image.rs)

**本地公开入口（同族）：** `alt`, `background_image`, `class`, `decorative`, `fallback_src`, `fill`, `fit`, `focus`, `height`, `image`, `name`, `node_ref`, `on`, `on_error`, `on_load`, `source`, `style`, `theme`, `unstyled`, `width`

**需要语义映射的属性候选：** `radius`, `src`

### Badge — M:Badge

来源：[@mantine/core](https://mantine.dev/core/badge/)；对应 `P3-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `children`, `circle`, `color`, `fullWidth`, `gradient`, `leftSection`, `radius`, `rightSection`, `size`

**官方功能章节：** Usage；Rounded；Left and right sections；Full width

**本地实现：** [src/display.rs](../src/display.rs)、[src/tag.rs](../src/tag.rs)

**本地公开入口（同族）：** `actions`, `badge`, `bordered`, `card`, `class`, `content`, `default_visible`, `description`, `empty_state`, `enabled`, `fill`, `footer`, `header`, `heading_level`, `illustration`, `name`, `node_ref`, `on_remove`, `on_remove_press`, `paper`, `pill`, `removable`, `remove_ref`, `shadow`, `size`, `style`, `theme`, `title`, `unstyled`, `variant`, `visible`

**需要语义映射的属性候选：** `autoContrast`, `children`, `circle`, `color`, `fullWidth`, `gradient`, `leftSection`, `radius`, `rightSection`

### Card — M:Card

来源：[@mantine/core](https://mantine.dev/core/card/)；对应 `P3-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `inheritPadding`, `orientation`, `padding`, `radius`, `shadow`, `withBorder`

**官方功能章节：** Usage；Horizontal orientation；Polymorphic component；Card.Section；Polymorphic Card.Section；withBorder and inheritPadding props

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/display.rs](../src/display.rs)

**本地公开入口（同族）：** `actions`, `aspect_ratio`, `badge`, `border_color`, `border_style`, `border_width`, `bordered`, `bottom`, `box_shadow`, `card`, `class`, `content`, `default_visible`, `description`, `display`, `empty_state`, `enabled`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `font_family`, `font_weight`, `footer`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `header`, `heading_level`, `height`, `illustration`, `is_finite`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `on_remove`, `on_remove_press`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `paper`, `pill`, `pointer_events`, `position`, `radius`, `removable`, `remove_ref`, `right`, `scroll_anchor`, `shadow`, `size`, `style`, `text_overflow`, `theme`, `title`, `top`, `unstyled`, `validate`, `variant`, `visible`, `white_space`, `width`

**需要语义映射的属性候选：** `children`, `inheritPadding`, `orientation`, `withBorder`

### ColorSwatch — M:ColorSwatch

来源：[@mantine/core](https://mantine.dev/core/color-swatch/)；对应 `P7-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `color`, `radius`, `size`, `withShadow`

**官方功能章节：** Usage；withShadow

**本地实现：** [src/color_swatch.rs](../src/color_swatch.rs)

**本地公开入口（同族）：** `class`, `color_swatch`, `default_pressed`, `enabled`, `interactive`, `name`, `node_ref`, `on`, `on_press`, `on_pressed_change`, `pressed`, `round`, `side`, `style`, `theme`, `unstyled`

**需要语义映射的属性候选：** `children`, `color`, `radius`, `size`, `withShadow`

### DataList — M:DataList

来源：[@mantine/core](https://mantine.dev/core/data-list/)；对应 `P3-18`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `gap`, `labelWidth`, `orientation`, `size`, `withDivider`

**官方功能章节：** Usage；With divider；Vertical orientation

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `children`, `gap`, `labelWidth`, `orientation`, `withDivider`

### Image — M:Image

来源：[@mantine/core](https://mantine.dev/core/image/)；对应 `P3-14`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `fallbackSrc`, `fit`, `onError`, `radius`, `src`

**官方功能章节：** Usage；Image height；Image fit；Fallback image；Usage with Next.js Image

**本地实现：** [src/image.rs](../src/image.rs)

**本地公开入口（同族）：** `alt`, `background_image`, `class`, `decorative`, `fallback_src`, `fill`, `fit`, `focus`, `height`, `image`, `name`, `node_ref`, `on`, `on_error`, `on_load`, `source`, `style`, `theme`, `unstyled`, `width`

**需要语义映射的属性候选：** `radius`, `src`

### Indicator — M:Indicator

来源：[@mantine/core](https://mantine.dev/core/indicator/)；对应 `P3-15`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `color`, `disabled`, `inline`, `label`, `maxValue`, `offset`, `position`, `processing`, `radius`, `showZero`, `size`, `withBorder`, `zIndex`

**官方功能章节：** Usage；Inline；Offset；Processing animation；Disabled；Max value；Show zero；Auto contrast

**本地实现：** [src/indicator.rs](../src/indicator.rs)、[src/theme_icon.rs](../src/theme_icon.rs)

**本地公开入口（同族）：** `class`, `color`, `diameter`, `fill`, `indicator`, `label`, `marker_ref`, `name`, `node_ref`, `position`, `size`, `style`, `theme`, `theme_icon`, `unstyled`, `variant`, `visible`

**需要语义映射的属性候选：** `autoContrast`, `disabled`, `inline`, `maxValue`, `offset`, `processing`, `radius`, `showZero`, `withBorder`, `zIndex`

### Kbd — M:Kbd

来源：[@mantine/core](https://mantine.dev/core/kbd/)；对应 `P3-16`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `size`

**官方功能章节：** Usage；Size

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### NumberFormatter — M:NumberFormatter

来源：[@mantine/core](https://mantine.dev/core/number-formatter/)；对应 `P3-20`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowNegative`, `decimalScale`, `decimalSeparator`, `fixedDecimalScale`, `prefix`, `suffix`, `thousandSeparator`, `thousandsGroupStyle`, `value`

**官方功能章节：** Usage；Prefix and suffix；Thousands separator；Decimal scale

**本地实现：** [src/number_formatter.rs](../src/number_formatter.rs)

**本地公开入口（同族）：** `class`, `format`, `grouping`, `locale`, `max_fraction_digits`, `min_fraction_digits`, `node_ref`, `number_formatter`, `size`, `style`, `theme`, `unstyled`, `value`

**需要语义映射的属性候选：** `allowNegative`, `decimalScale`, `decimalSeparator`, `fixedDecimalScale`, `prefix`, `suffix`, `thousandSeparator`, `thousandsGroupStyle`

### OverflowList — M:OverflowList

来源：[@mantine/core](https://mantine.dev/core/overflow-list/)；对应 `P6-12`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `collapseFrom`, `data`, `gap`, `getItemKey`, `maxRows`, `maxVisibleItems`, `renderItem`, `renderOverflow`

**官方功能章节：** Usage；Data type；Max rows；Max visible items；Collapse from；Example: overflow with hover card

**本地实现：** [src/overflow_list/mod.rs](../src/overflow_list/mod.rs)

**本地公开入口（同族）：** `class`, `content`, `dynamic_items`, `enabled`, `focus`, `gap`, `handle_ref`, `initial_visible`, `item`, `items`, `measurement_revision`, `name`, `node_ref`, `on`, `on_open_change`, `on_visible_count_change`, `open`, `overflow`, `overflow_count`, `overflow_item`, `overflow_label`, `overflow_list`, `pending_queries`, `refresh`, `set_open`, `size`, `style`, `theme`, `unstyled`, `visible_count`, `width`

**需要语义映射的属性候选：** `collapseFrom`, `data`, `getItemKey`, `maxRows`, `maxVisibleItems`, `renderItem`, `renderOverflow`

### RollingNumber — M:RollingNumber

来源：[@mantine/core](https://mantine.dev/core/rolling-number/)；对应 `P6-14`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `animationDuration`, `decimalScale`, `decimalSeparator`, `fixedDecimalScale`, `prefix`, `suffix`, `tabularNumbers`, `thousandSeparator`, `timingFunction`, `value`, `withLiveRegion`

**官方功能章节：** Usage；Prefix and suffix；Animation duration；Text selection and copying

**本地实现：** [../glory/crates/blitz/src/translation.rs](../../glory/crates/blitz/src/translation.rs)、[../glory/crates/render/src/translation.rs](../../glory/crates/render/src/translation.rs)、[../glory/crates/web/src/browser/translation.rs](../../glory/crates/web/src/browser/translation.rs)、[src/marquee.rs](../src/marquee.rs)、[src/rolling_number.rs](../src/rolling_number.rs)

**本地公开入口（同族）：** `admitted`, `animated`, `class`, `config`, `configure`, `direction`, `duration_millis`, `format`, `gap`, `grouping`, `is_empty`, `locale`, `marquee`, `max_fraction_digits`, `measurement_revision`, `min_fraction_digits`, `new`, `node_ref`, `offset`, `on`, `origin`, `paused`, `period_millis`, `resting_offset`, `retire`, `rolling_number`, `running`, `size`, `style`, `text`, `theme`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `animationDuration`, `decimalScale`, `decimalSeparator`, `fixedDecimalScale`, `prefix`, `suffix`, `tabularNumbers`, `thousandSeparator`, `timingFunction`, `withLiveRegion`

### Spoiler — M:Spoiler

来源：[@mantine/core](https://mantine.dev/core/spoiler/)；对应 `P3-21`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `controlRef`, `defaultExpanded`, `expanded`, `hideAriaLabel`, `hideLabel`, `maxHeight`, `onExpandedChange`, `showAriaLabel`, `showLabel`, `transitionDuration`

**官方功能章节：** Usage；Control expanded state；Subscribe to expanded state changes；Transition duration；Get control ref；Accessibility

**本地实现：** [src/spoiler.rs](../src/spoiler.rs)

**本地公开入口（同族）：** `class`, `content`, `default_expanded`, `enabled`, `expanded`, `focus`, `labels`, `max_height`, `name`, `node_ref`, `on`, `on_expanded_change`, `size`, `spoiler`, `style`, `theme`, `trigger_ref`, `unstyled`, `viewport_ref`

**需要语义映射的属性候选：** `controlRef`, `hideAriaLabel`, `hideLabel`, `showAriaLabel`, `showLabel`, `transitionDuration`

### ThemeIcon — M:ThemeIcon

来源：[@mantine/core](https://mantine.dev/core/theme-icon/)；对应 `P3-15`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `children`, `color`, `gradient`, `radius`, `size`

**官方功能章节：** Usage；Customize variants colors

**本地实现：** [src/indicator.rs](../src/indicator.rs)、[src/theme_icon.rs](../src/theme_icon.rs)

**本地公开入口（同族）：** `class`, `color`, `diameter`, `fill`, `indicator`, `label`, `marker_ref`, `name`, `node_ref`, `position`, `size`, `style`, `theme`, `theme_icon`, `unstyled`, `variant`, `visible`

**需要语义映射的属性候选：** `autoContrast`, `children`, `gradient`, `radius`

### Timeline — M:Timeline

来源：[@mantine/core](https://mantine.dev/core/timeline/)；对应 `P3-19`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `align`, `alternate`, `autoContrast`, `bullet`, `bulletSize`, `children`, `color`, `lineVariant`, `lineWidth`, `opposite`, `radius`, `reverseActive`, `title`

**官方功能章节：** Usage；Line and bullet props；Opposite content；Alternate items；Bullet as React node；Wrap Timeline.Item

**本地实现：** [src/separator.rs](../src/separator.rs)、[src/timeline.rs](../src/timeline.rs)

**本地公开入口（同族）：** `active`, `class`, `decorative`, `description`, `fill`, `heading_level`, `item`, `items`, `length`, `marker`, `name`, `node_ref`, `orientation`, `progress`, `separator`, `size`, `state_labels`, `style`, `theme`, `thickness`, `timeline`, `timeline_item`, `timestamp`, `title`, `unstyled`

**需要语义映射的属性候选：** `align`, `alternate`, `autoContrast`, `bullet`, `bulletSize`, `children`, `color`, `lineVariant`, `lineWidth`, `opposite`, `radius`, `reverseActive`

### Blockquote — M:Blockquote

来源：[@mantine/core](https://mantine.dev/core/blockquote/)；对应 `P3-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `cite`, `color`, `icon`, `iconSize`, `radius`, `textWrap`

**官方功能章节：** Usage；Text wrap

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `cite`, `color`, `icon`, `iconSize`, `radius`, `textWrap`

### Code — M:Code

来源：[@mantine/core](https://mantine.dev/core/code/)；对应 `P3-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `activeTab`, `background`, `block`, `code`, `codeColorScheme`, `collapseCodeLabel`, `color`, `controls`, `copiedLabel`, `copyLabel`, `defaultActiveTab`, `defaultExpanded`, `expandCodeLabel`, `expanded`, `getFileIcon`, `language`, `maxCollapsedHeight`, `onExpandedChange`, `onTabChange`, `radius`, `withBorder`, `withCopyButton`, `withExpandButton`, `withFirstLineIndentation`, `withLineNumbers`

**官方功能章节：** Usage；Block code；Custom color

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `activeTab`, `background`, `block`, `code`, `codeColorScheme`, `collapseCodeLabel`, `color`, `controls`, `copiedLabel`, `copyLabel`, `defaultActiveTab`, `defaultExpanded`, `expandCodeLabel`, `expanded`, `getFileIcon`, `language`, `maxCollapsedHeight`, `onExpandedChange`, `onTabChange`, `radius`, `withBorder`, `withCopyButton`, `withExpandButton`, `withFirstLineIndentation`, `withLineNumbers`

### Highlight — M:Highlight

来源：[@mantine/core](https://mantine.dev/core/highlight/)；对应 `P3-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accentInsensitive`, `caseInsensitive`, `children`, `color`, `gradient`, `highlight`, `highlightStyles`, `inherit`, `inline`, `lineClamp`, `size`, `span`, `textWrap`, `truncate`, `wholeWord`

**官方功能章节：** Usage；Matching behavior；Case-sensitive matching；Accent-sensitive matching；Highlight multiple substrings；Custom colors per term；Whole word matching；Change highlight styles；Text props

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `accentInsensitive`, `caseInsensitive`, `children`, `color`, `gradient`, `highlightStyles`, `inherit`, `inline`, `lineClamp`, `span`, `textWrap`, `truncate`, `wholeWord`

### List — M:List

来源：[@mantine/core](https://mantine.dev/core/list/)；对应 `P3-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `center`, `children`, `icon`, `listStyleType`, `reversed`, `size`, `spacing`, `start`, `type`, `withPadding`

**官方功能章节：** Usage；With icons；Nested lists；Ordered list numbering

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `center`, `children`, `icon`, `listStyleType`, `reversed`, `spacing`, `start`, `type`, `withPadding`

### Mark — M:Mark

来源：[@mantine/core](https://mantine.dev/core/mark/)；对应 `P3-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`

**官方功能章节：** Usage

**本地实现：** [../glory/crates/blitz/src/accessibility.rs](../../glory/crates/blitz/src/accessibility.rs)、[../glory/crates/html/src/mapping.rs](../../glory/crates/html/src/mapping.rs)、[../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/document.rs](../src/document.rs)

**本地公开入口（同族）：** `actions`, `blockquote`, `citation`, `class`, `content`, `data_list`, `data_list_item`, `description`, `fill`, `highlight`, `item`, `items`, `kbd`, `key`, `media`, `name`, `node_ref`, `ordered`, `platform`, `preflight`, `queries`, `query`, `shape`, `shape_in`, `size`, `style`, `term`, `text_list`, `text_list_item`, `theme`, `title`, `unstyled`

**需要语义映射的属性候选：** `color`

### Table — M:Table

来源：[@mantine/core](https://mantine.dev/core/table/)；对应 `P8-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `borderColor`, `captionSide`, `color`, `data`, `depthOffset`, `getControlProps`, `highlightOnHover`, `highlightOnHoverColor`, `horizontalSpacing`, `initialData`, `layout`, `minDepthToOffset`, `radius`, `reinitializeRef`, `scrollSpyOptions`, `size`, `stickyHeader`, `stickyHeaderOffset`, `striped`, `stripedColor`, `tabularNums`, `verticalSpacing`, `withColumnBorders`, `withRowBorders`, `withTableBorder`

**官方功能章节：** Usage；data prop；Sticky header；Spacing；Caption and tfoot；Striped and rows hover；Scroll container；Vertical variant；Tabular numbers；Example: Table virtualization；Example: Table with row selection

**本地实现：** [../glory/crates/render/src/document.rs](../../glory/crates/render/src/document.rs)、[src/table/mod.rs](../src/table/mod.rs)、[src/table/model.rs](../src/table/model.rs)

**本地公开入口（同族）：** `body`, `bordered`, `caption`, `cell`, `cells`, `class`, `column_span`, `content`, `foot`, `head`, `name`, `node_ref`, `row`, `row_span`, `rows`, `size`, `striped`, `style`, `table`, `table_body`, `table_cell`, `table_foot`, `table_head`, `table_header_cell`, `table_row`, `table_row_header`, `text`, `theme`, `unstyled`

**需要语义映射的属性候选：** `autoContrast`, `borderColor`, `captionSide`, `color`, `data`, `depthOffset`, `getControlProps`, `highlightOnHover`, `highlightOnHoverColor`, `horizontalSpacing`, `initialData`, `layout`, `minDepthToOffset`, `radius`, `reinitializeRef`, `scrollSpyOptions`, `stickyHeader`, `stickyHeaderOffset`, `stripedColor`, `tabularNums`, `verticalSpacing`, `withColumnBorders`, `withRowBorders`, `withTableBorder`

### Text — M:Text

来源：[@mantine/core](https://mantine.dev/core/text/)；对应 `P3-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autosize`, `bottomSection`, `bottomSectionProps`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `gradient`, `inherit`, `inline`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `lineClamp`, `loading`, `loadingPosition`, `maxRows`, `minRows`, `pointer`, `radius`, `required`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `span`, `success`, `successProps`, `textWrap`, `truncate`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Truncate；Line clamp；Inherit styles；Text wrap；span prop

**本地实现：** [src/typography.rs](../src/typography.rs)

**本地公开入口（同族）：** `anchor`, `class`, `color`, `content`, `fill`, `font_size`, `inline`, `line_height`, `name`, `node_ref`, `on`, `selectable`, `size`, `style`, `text`, `theme`, `title`, `truncate`, `typography`, `unstyled`, `variant`, `weight`

**需要语义映射的属性候选：** `autosize`, `bottomSection`, `bottomSectionProps`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `gradient`, `inherit`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `lineClamp`, `loading`, `loadingPosition`, `maxRows`, `minRows`, `pointer`, `radius`, `required`, `resize`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `span`, `success`, `successProps`, `textWrap`, `withAsterisk`, `withErrorStyles`, `withSuccessStyles`, `wrapperProps`

### Title — M:Title

来源：[@mantine/core](https://mantine.dev/core/title/)；对应 `P3-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `lineClamp`, `order`, `size`, `textWrap`

**官方功能章节：** Usage；Size；Text wrap；Line clamp

**本地实现：** [src/typography.rs](../src/typography.rs)

**本地公开入口（同族）：** `anchor`, `class`, `color`, `content`, `fill`, `font_size`, `inline`, `line_height`, `name`, `node_ref`, `on`, `selectable`, `size`, `style`, `text`, `theme`, `title`, `truncate`, `typography`, `unstyled`, `variant`, `weight`

**需要语义映射的属性候选：** `lineClamp`, `order`, `textWrap`

### Typography — M:Typography

来源：[@mantine/core](https://mantine.dev/core/typography/)；对应 `P3-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Usage；Example；All styles demo

**本地实现：** [src/typography.rs](../src/typography.rs)

**本地公开入口（同族）：** `anchor`, `class`, `color`, `content`, `fill`, `font_size`, `inline`, `line_height`, `name`, `node_ref`, `on`, `selectable`, `size`, `style`, `text`, `theme`, `title`, `truncate`, `typography`, `unstyled`, `variant`, `weight`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Box — M:Box

来源：[@mantine/core](https://mantine.dev/core/box/)；对应 `P3-01`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Usage

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/layout.rs](../src/layout.rs)

**本地公开入口（同族）：** `align`, `aspect_ratio`, `border_color`, `border_style`, `border_width`, `bottom`, `box_shadow`, `class`, `container`, `direction`, `display`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `fluid`, `font_family`, `font_weight`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `height`, `is_finite`, `justify`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `pointer_events`, `position`, `radius`, `right`, `scroll_anchor`, `size`, `style`, `text_overflow`, `theme`, `top`, `unstyled`, `validate`, `white_space`, `width`, `wrap`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Collapse — M:Collapse

来源：[@mantine/core](https://mantine.dev/core/collapse/)；对应 `P0-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `animateOpacity`, `expanded`, `keepMounted`, `keepMountedMode`, `onTransitionEnd`, `onTransitionStart`, `orientation`, `transitionDuration`, `transitionTimingFunction`

**官方功能章节：** Usage；Horizontal orientation；Change transition；Example: nested Collapse components；use-collapse hook

**本地实现：** [../glory/crates/blitz/src/collapse.rs](../../glory/crates/blitz/src/collapse.rs)、[../glory/crates/render/src/collapse.rs](../../glory/crates/render/src/collapse.rs)、[../glory/crates/web/src/browser/collapse.rs](../../glory/crates/web/src/browser/collapse.rs)、[src/collapsible/mod.rs](../src/collapsible/mod.rs)

**本地公开入口（同族）：** `animated`, `class`, `collapsible`, `content`, `content_policy`, `default_open`, `enabled`, `focus`, `height_transition`, `is_empty`, `label`, `name`, `needs_frame`, `node_ref`, `on`, `on_open_change`, `open`, `panel_ref`, `ready`, `retire`, `running`, `size`, `style`, `theme`, `trigger_content`, `trigger_ref`, `unstyled`, `update`, `validate`

**需要语义映射的属性候选：** `animateOpacity`, `expanded`, `keepMounted`, `keepMountedMode`, `onTransitionEnd`, `onTransitionStart`, `orientation`, `transitionDuration`, `transitionTimingFunction`

### Divider — M:Divider

来源：[@mantine/core](https://mantine.dev/core/divider/)；对应 `P3-12`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `color`, `label`, `labelPosition`, `orientation`, `size`

**官方功能章节：** Usage；Variants；With label；Sizes；Vertical orientation

**本地实现：** [src/separator.rs](../src/separator.rs)、[src/timeline.rs](../src/timeline.rs)

**本地公开入口（同族）：** `active`, `class`, `decorative`, `description`, `fill`, `heading_level`, `item`, `items`, `length`, `marker`, `name`, `node_ref`, `orientation`, `progress`, `separator`, `size`, `state_labels`, `style`, `theme`, `thickness`, `timeline`, `timeline_item`, `timestamp`, `title`, `unstyled`

**需要语义映射的属性候选：** `color`, `label`, `labelPosition`

### FocusTrap — M:FocusTrap

来源：[@mantine/core](https://mantine.dev/core/focus-trap/)；对应 `P0-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `children`, `innerRef`, `refProp`

**官方功能章节：** Usage；Initial focus；FocusTrap.InitialFocus；Focus trapping logic

**本地实现：** [../glory/crates/blitz/src/focus_scope.rs](../../glory/crates/blitz/src/focus_scope.rs)、[../glory/crates/render/src/focus_scope.rs](../../glory/crates/render/src/focus_scope.rs)、[../glory/crates/ui/src/app.rs](../../glory/crates/ui/src/app.rs)、[../glory/crates/web/src/browser/focus_scope.rs](../../glory/crates/web/src/browser/focus_scope.rs)、[src/focus_scope.rs](../src/focus_scope.rs)、[src/portal.rs](../src/portal.rs)

**本地公开入口（同族）：** `action`, `active`, `auto_focus`, `batch`, `buffer_stats`, `cancel_request`, `capabilities`, `child`, `child_in`, `class`, `clear`, `clear_files`, `configuration`, `contain_focus`, `contains_node`, `content`, `copy_text`, `create`, `defer_descendant_disposal`, `dispatch`, `downgrade`, `enabled`, `fill`, `flush`, `focus_scope`, `focus_scope_nodes`, `focus_scope_roots`, `focus_scopes`, `graph_stats`, `initial_focus`, `instantiate`, `is_active`, `is_enabled`, `is_inert`, `is_within`, `link_request`, `loop_tab`, `memory`, `mount`, `mount_with`, `new`, `node_ref`, `on`, `on_target_lost`, `patch`, `pick_files`, `place`, `place_before`, `portal`, `portal_context`, `portal_target`, `profile`, `pump`, `remove_file`, `report`, `request`, `restore_focus`, `retain`, `return_focus`, `rich_editor_editable`, `rich_editor_value`, `roots`, `runtime`, `runtime_stats`, `scope`, `semantic_id`, `set_file_configuration`, `set_input_selection`, `set_profile`, `set_rich_document`, `set_rich_selection`, `settle`, `sources`, `stats`, `style`, `style_profile`, `suspend`, `targets`, `tree`, `upgrade`, `with_host`

**需要语义映射的属性候选：** `children`, `innerRef`, `refProp`

### Marquee — M:Marquee

来源：[@mantine/core](https://mantine.dev/core/marquee/)；对应 `P6-14`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `duration`, `fadeEdgeColor`, `fadeEdgeSize`, `fadeEdges`, `gap`, `orientation`, `pauseOnHover`, `repeat`, `reverse`

**官方功能章节：** Usage；Pause on hover；Vertical orientation；Multiple rows；Fade edges；Customization

**本地实现：** [../glory/crates/blitz/src/translation.rs](../../glory/crates/blitz/src/translation.rs)、[../glory/crates/render/src/translation.rs](../../glory/crates/render/src/translation.rs)、[../glory/crates/web/src/browser/translation.rs](../../glory/crates/web/src/browser/translation.rs)、[src/marquee.rs](../src/marquee.rs)、[src/rolling_number.rs](../src/rolling_number.rs)

**本地公开入口（同族）：** `admitted`, `animated`, `class`, `config`, `configure`, `direction`, `duration_millis`, `format`, `gap`, `grouping`, `is_empty`, `locale`, `marquee`, `max_fraction_digits`, `measurement_revision`, `min_fraction_digits`, `new`, `node_ref`, `offset`, `on`, `origin`, `paused`, `period_millis`, `resting_offset`, `retire`, `rolling_number`, `running`, `size`, `style`, `text`, `theme`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `children`, `duration`, `fadeEdgeColor`, `fadeEdgeSize`, `fadeEdges`, `orientation`, `pauseOnHover`, `repeat`, `reverse`

### Paper — M:Paper

来源：[@mantine/core](https://mantine.dev/core/paper/)；对应 `P3-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `radius`, `shadow`, `withBorder`

**官方功能章节：** Usage

**本地实现：** [../glory/crates/style/src/lib.rs](../../glory/crates/style/src/lib.rs)、[src/display.rs](../src/display.rs)

**本地公开入口（同族）：** `actions`, `aspect_ratio`, `badge`, `border_color`, `border_style`, `border_width`, `bordered`, `bottom`, `box_shadow`, `card`, `class`, `content`, `default_visible`, `description`, `display`, `empty_state`, `enabled`, `field_sizing`, `fill`, `flex_direction`, `flex_wrap`, `font_family`, `font_weight`, `footer`, `gap`, `grid_column`, `grid_columns`, `grid_row`, `grid_rows`, `header`, `heading_level`, `height`, `illustration`, `is_finite`, `left`, `margin`, `max_height`, `max_width`, `min_height`, `min_width`, `name`, `new`, `node_ref`, `object_fit`, `on_remove`, `on_remove_press`, `opacity`, `overflow_x`, `overflow_y`, `padding`, `paper`, `pill`, `pointer_events`, `position`, `radius`, `removable`, `remove_ref`, `right`, `scroll_anchor`, `shadow`, `size`, `style`, `text_overflow`, `theme`, `title`, `top`, `unstyled`, `validate`, `variant`, `visible`, `white_space`, `width`

**需要语义映射的属性候选：** `withBorder`

### Portal — M:Portal

来源：[@mantine/core](https://mantine.dev/core/portal/)；对应 `P0-11`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `reuseTargetNode`, `target`

**官方功能章节：** Usage；Reuse target node；Specify target DOM node；Server side rendering；OptionalPortal component

**本地实现：** [../glory/crates/blitz/src/focus_scope.rs](../../glory/crates/blitz/src/focus_scope.rs)、[../glory/crates/render/src/focus_scope.rs](../../glory/crates/render/src/focus_scope.rs)、[../glory/crates/ui/src/app.rs](../../glory/crates/ui/src/app.rs)、[../glory/crates/web/src/browser/focus_scope.rs](../../glory/crates/web/src/browser/focus_scope.rs)、[src/focus_scope.rs](../src/focus_scope.rs)、[src/portal.rs](../src/portal.rs)

**本地公开入口（同族）：** `action`, `active`, `auto_focus`, `batch`, `buffer_stats`, `cancel_request`, `capabilities`, `child`, `child_in`, `class`, `clear`, `clear_files`, `configuration`, `contain_focus`, `contains_node`, `content`, `copy_text`, `create`, `defer_descendant_disposal`, `dispatch`, `downgrade`, `enabled`, `fill`, `flush`, `focus_scope`, `focus_scope_nodes`, `focus_scope_roots`, `focus_scopes`, `graph_stats`, `initial_focus`, `instantiate`, `is_active`, `is_enabled`, `is_inert`, `is_within`, `link_request`, `loop_tab`, `memory`, `mount`, `mount_with`, `new`, `node_ref`, `on`, `on_target_lost`, `patch`, `pick_files`, `place`, `place_before`, `portal`, `portal_context`, `portal_target`, `profile`, `pump`, `remove_file`, `report`, `request`, `restore_focus`, `retain`, `return_focus`, `rich_editor_editable`, `rich_editor_value`, `roots`, `runtime`, `runtime_stats`, `scope`, `semantic_id`, `set_file_configuration`, `set_input_selection`, `set_profile`, `set_rich_document`, `set_rich_selection`, `settle`, `sources`, `stats`, `style`, `style_profile`, `suspend`, `targets`, `tree`, `upgrade`, `with_host`

**需要语义映射的属性候选：** `children`, `reuseTargetNode`, `target`

### ScrollArea — M:ScrollArea

来源：[@mantine/core](https://mantine.dev/core/scroll-area/)；对应 `P6-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `offsetScrollbars`, `onBottomReached`, `onLeftReached`, `onOverflowChange`, `onRightReached`, `onScrollPositionChange`, `onTopReached`, `overscrollBehavior`, `scrollHideDelay`, `scrollbarSize`, `scrollbars`, `startScrollPosition`, `type`, `verticalScrollbarPosition`, `viewportProps`, `viewportRef`

**官方功能章节：** Usage；Offset size；Horizontal scrollbars；Disable horizontal scrollbars；Vertical scrollbar position；Subscribe to scroll position changes；Scroll boundary callbacks；Scroll to position；Start scroll position；Styles API；Scroll element into view；ScrollArea.Autosize；ScrollArea.Autosize with Popover

**本地实现：** [../glory/crates/render/src/scroll.rs](../../glory/crates/render/src/scroll.rs)、[src/scroll_area.rs](../src/scroll_area.rs)

**本地公开入口（同族）：** `allows`, `area`, `as_str`, `axes`, `axis`, `class`, `command`, `content`, `content_ref`, `key`, `keyboard`, `metrics`, `name`, `node_ref`, `on`, `on_metrics_change`, `on_ready`, `on_scroll_change`, `refresh`, `scroll_area`, `scroll_by`, `scroll_to`, `scrollbars`, `scroller`, `size`, `step`, `style`, `theme`, `unstyled`, `validate`

**需要语义映射的属性候选：** `offsetScrollbars`, `onBottomReached`, `onLeftReached`, `onOverflowChange`, `onRightReached`, `onScrollPositionChange`, `onTopReached`, `overscrollBehavior`, `scrollHideDelay`, `scrollbarSize`, `startScrollPosition`, `type`, `verticalScrollbarPosition`, `viewportProps`, `viewportRef`

### Scroller — M:Scroller

来源：[@mantine/core](https://mantine.dev/core/scroller/)；对应 `P6-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `controlSize`, `draggable`, `edgeGradientColor`, `endControlIcon`, `endControlProps`, `scrollAmount`, `showEndControl`, `showStartControl`, `startControlIcon`, `startControlProps`

**官方功能章节：** Usage；Mouse drag scrolling；Scroll amount；Control size；Custom icons；Usage with Tabs

**本地实现：** [../glory/crates/render/src/scroll.rs](../../glory/crates/render/src/scroll.rs)、[src/scroll_area.rs](../src/scroll_area.rs)

**本地公开入口（同族）：** `allows`, `area`, `as_str`, `axes`, `axis`, `class`, `command`, `content`, `content_ref`, `key`, `keyboard`, `metrics`, `name`, `node_ref`, `on`, `on_metrics_change`, `on_ready`, `on_scroll_change`, `refresh`, `scroll_area`, `scroll_by`, `scroll_to`, `scrollbars`, `scroller`, `size`, `step`, `style`, `theme`, `unstyled`, `validate`

**需要语义映射的属性候选：** `children`, `controlSize`, `draggable`, `edgeGradientColor`, `endControlIcon`, `endControlProps`, `scrollAmount`, `showEndControl`, `showStartControl`, `startControlIcon`, `startControlProps`

### Toolbar — M:Toolbar

来源：[@mantine/core](https://mantine.dev/core/toolbar/)；对应 `P6-10`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `active`, `autoContrast`, `autoWidth`, `children`, `color`, `defaultValue`, `disabled`, `loop`, `onChange`, `orientation`, `radius`, `size`, `type`, `value`, `withBorder`

**官方功能章节：** Usage；Toggle groups；Size, orientation, variant and color；Auto contrast；Auto width；Disabled state；Accessibility；Keyboard interactions

**本地实现：** [src/toolbar.rs](../src/toolbar.rs)

**本地公开入口（同族）：** `action_bar`, `button`, `buttons`, `class`, `clear_label`, `direction`, `enabled`, `group`, `groups`, `loop_focus`, `name`, `node_ref`, `on`, `on_clear_selection`, `orientation`, `selected_count`, `size`, `style`, `theme`, `toolbar`, `toolbar_group`, `unstyled`, `visible`

**需要语义映射的属性候选：** `active`, `autoContrast`, `autoWidth`, `children`, `color`, `defaultValue`, `disabled`, `loop`, `onChange`, `radius`, `type`, `value`, `withBorder`

### Transition — M:Transition

来源：[@mantine/core](https://mantine.dev/core/transition/)；对应 `P0-12`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `duration`, `enterDelay`, `exitDelay`, `exitDuration`, `keepMounted`, `keepMountedMode`, `mounted`, `onEnter`, `onEntered`, `onExit`, `onExited`, `timingFunction`, `transition`

**官方功能章节：** Usage；Premade transitions；Custom transitions；Enter and exit delay；Reduced motion；Lifecycle callbacks；Keep mounted

**本地实现：** [src/environment.rs](../src/environment.rs)、[src/theme/mod.rs](../src/theme/mod.rs)、[src/theme_tokens.rs](../src/theme_tokens.rs)、[src/typography.rs](../src/typography.rs)

**本地公开入口（同族）：** `anchor`, `arabic`, `blue_dark`, `blue_light`, `chinese`, `class`, `color`, `content`, `control`, `dark`, `direction`, `direction_provider`, `english`, `fill`, `font_size`, `from_default`, `green_dark`, `green_light`, `inline`, `language`, `light`, `line_height`, `locale`, `locale_provider`, `message`, `motion`, `name`, `new`, `node_ref`, `on`, `palette`, `provider`, `selectable`, `size`, `spacing`, `style`, `stylesheet`, `text`, `theme`, `theme_provider`, `title`, `transition_duration`, `truncate`, `typography`, `unstyled`, `validate`, `variant`, `weight`, `with_control`, `with_spacing`, `with_tokens`, `with_typography`

**需要语义映射的属性候选：** `children`, `duration`, `enterDelay`, `exitDelay`, `exitDuration`, `keepMounted`, `keepMountedMode`, `mounted`, `onEnter`, `onEntered`, `onExit`, `onExited`, `timingFunction`, `transition`

### VisuallyHidden — M:VisuallyHidden

来源：[@mantine/core](https://mantine.dev/core/visually-hidden/)；对应 `P0-13`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Usage；HOOKS COMPONENTS AND FEATURES

**本地实现：** [src/visually_hidden.rs](../src/visually_hidden.rs)

**本地公开入口（同族）：** `class`, `content`, `node_ref`, `semantic_id`, `visually_hidden`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### DatePicker — D:DatePicker

来源：[@mantine/dates](https://mantine.dev/dates/date-picker/)；对应 `P9-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `enableKeyboardNavigation`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** Usage；Allow deselect；Multiple dates；Dates range；Single date in range；Presets；Default date；Controlled date；Default level；Hide outside dates；Display week numbers；First day of week；Hide weekdays；Weekend days；Render day function；Min and max date；Change header controls order；Add props to day, year and month control；Exclude dates；Number of columns；Max level；Full width；Size；Change year and months controls format；Change label format；Localization；Native level select；Accessibility

**本地实现：** [src/date_picker.rs](../src/date_picker.rs)、[src/date_picker_input.rs](../src/date_picker_input.rs)、[src/date_selection.rs](../src/date_selection.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `calendar_ref`, `class`, `close_label`, `contains`, `date_picker`, `date_picker_input`, `default_month`, `default_open`, `default_value`, `direction`, `disable_weekday`, `enabled`, `exclude_date`, `exclude_dates`, `field`, `focus`, `form_name`, `format_iso`, `insert`, `invalid`, `is_complete`, `iter`, `mode`, `month`, `name`, `node_ref`, `on`, `on_month_change`, `on_open_change`, `on_reject`, `on_value_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `enableKeyboardNavigation`, `error`, `errorProps`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### DatePickerInput — D:DatePickerInput

来源：[@mantine/dates](https://mantine.dev/dates/date-picker-input/)；对应 `P9-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `enableKeyboardNavigation`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** DatePicker props；Usage；Multiple dates；Dates range；Presets；Open picker in modal；Value format；Value formatter；Clearable；Disabled state；Min and max dates；Default level；Max level；Control props；Exclude dates；Hide outside dates；Hide weekdays；Weekend days；First day of week；Label format；List format；With week numbers；Render day；Header controls order；Input props；With icon；Native level select；Accessibility

**本地实现：** [src/date_picker.rs](../src/date_picker.rs)、[src/date_picker_input.rs](../src/date_picker_input.rs)、[src/date_selection.rs](../src/date_selection.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `calendar_ref`, `class`, `close_label`, `contains`, `date_picker`, `date_picker_input`, `default_month`, `default_open`, `default_value`, `direction`, `disable_weekday`, `enabled`, `exclude_date`, `exclude_dates`, `field`, `focus`, `form_name`, `format_iso`, `insert`, `invalid`, `is_complete`, `iter`, `mode`, `month`, `name`, `node_ref`, `on`, `on_month_change`, `on_open_change`, `on_reject`, `on_value_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `enableKeyboardNavigation`, `error`, `errorProps`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### DateTimePicker — D:DateTimePicker

来源：[@mantine/dates](https://mantine.dev/dates/date-time-picker/)；对应 `P9-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultTimeValue`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `endTimePickerProps`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `submitButtonProps`, `success`, `successProps`, `timePickerProps`, `type`, `value`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSeconds`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** DatePicker props；Usage；Range；With seconds；Presets；TimePicker props；Value format；Disabled state；Input props；Clearable；Open picker in modal；Native level select；Accessibility

**本地实现：** [src/date_time_model.rs](../src/date_time_model.rs)、[src/date_time_picker.rs](../src/date_time_picker.rs)

**本地公开入口（同族）：** `application`, `bounds`, `calendar_ref`, `class`, `classify`, `contains`, `date_bounds`, `default_value`, `direction`, `enabled`, `field`, `focus`, `format`, `format_local_date_time`, `from_fixed`, `from_local`, `grid`, `hour_ref`, `inline_date_time_picker`, `is_error`, `minute_ref`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_resolved`, `on_value_change`, `parse_local_date_time`, `proposal`, `provider`, `readonly`, `reset_revision`, `resolve`, `second_ref`, `size`, `style`, `theme`, `time_bounds`, `unstyled`, `value`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultTimeValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `endTimePickerProps`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousIcon`, `previousLabel`, `radius`, `readOnly`, `renderDay`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `submitButtonProps`, `success`, `successProps`, `timePickerProps`, `type`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSeconds`, `withSuccessStyles`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### InlineDateTimePicker — D:InlineDateTimePicker

来源：[@mantine/dates](https://mantine.dev/dates/inline-date-time-picker/)；对应 `P9-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultTimeValue`, `defaultValue`, `enableKeyboardNavigation`, `endTimePickerProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `labelSeparator`, `level`, `locale`, `maxDate`, `maxLevel`, `minDate`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onSubmit`, `onYearMouseEnter`, `onYearSelect`, `presets`, `previousIcon`, `previousLabel`, `renderDay`, `size`, `submitButtonProps`, `timePickerProps`, `type`, `value`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withCellSpacing`, `withNativeLevelSelect`, `withSeconds`, `withWeekNumbers`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** DatePicker props；Usage；With seconds；Range；Controlled range；Value format

**本地实现：** [src/date_time_model.rs](../src/date_time_model.rs)、[src/date_time_picker.rs](../src/date_time_picker.rs)

**本地公开入口（同族）：** `application`, `bounds`, `calendar_ref`, `class`, `classify`, `contains`, `date_bounds`, `default_value`, `direction`, `enabled`, `field`, `focus`, `format`, `format_local_date_time`, `from_fixed`, `from_local`, `grid`, `hour_ref`, `inline_date_time_picker`, `is_error`, `minute_ref`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_resolved`, `on_value_change`, `parse_local_date_time`, `proposal`, `provider`, `readonly`, `reset_revision`, `resolve`, `second_ref`, `size`, `style`, `theme`, `time_bounds`, `unstyled`, `value`

**需要语义映射的属性候选：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultTimeValue`, `enableKeyboardNavigation`, `endTimePickerProps`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `labelSeparator`, `level`, `locale`, `maxDate`, `maxLevel`, `minDate`, `monthLabelFormat`, `monthsListFormat`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onLevelChange`, `onMonthMouseEnter`, `onMonthSelect`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onSubmit`, `onYearMouseEnter`, `onYearSelect`, `presets`, `previousIcon`, `previousLabel`, `renderDay`, `submitButtonProps`, `timePickerProps`, `type`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withCellSpacing`, `withNativeLevelSelect`, `withSeconds`, `withWeekNumbers`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### DateInput — D:DateInput

来源：[@mantine/dates](https://mantine.dev/dates/date-input/)；对应 `P9-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `columnsToScroll`, `date`, `dateParser`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fixOnBlur`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `hasNextLevel`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `monthLabelFormat`, `monthsListFormat`, `nextDisabled`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onLevelChange`, `onLevelClick`, `onNext`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPrevious`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `pointer`, `popoverProps`, `presets`, `previousDisabled`, `previousIcon`, `previousLabel`, `radius`, `renderDay`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `value`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withNext`, `withPrevious`, `withSuccessStyles`, `withTime`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** DatePicker props；Usage；Presets；Value format；With time；Date parser；Allow clear；Min and max date；Disabled state；Input props；Accessibility

**本地实现：** [src/date_draft.rs](../src/date_draft.rs)、[src/date_input.rs](../src/date_input.rs)

**本地公开入口（同族）：** `class`, `classify_date_draft`, `date_input`, `default_value`, `enabled`, `field`, `focus`, `form_name`, `invalid`, `is_error`, `name`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_value_change`, `parse_iso_date_draft`, `parser`, `picker_close_label`, `picker_label`, `picker_title`, `proposal`, `readonly`, `required`, `size`, `style`, `theme`, `unstyled`, `value`, `with_picker`

**需要语义映射的属性候选：** `allowDeselect`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `columnsToScroll`, `date`, `dateParser`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `excludeDate`, `firstDayOfWeek`, `fixOnBlur`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `hasNextLevel`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `monthLabelFormat`, `monthsListFormat`, `nextDisabled`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onLevelChange`, `onLevelClick`, `onNext`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPrevious`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `pointer`, `popoverProps`, `presets`, `previousDisabled`, `previousIcon`, `previousLabel`, `radius`, `renderDay`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `valueFormat`, `weekdayFormat`, `weekendDays`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withNext`, `withPrevious`, `withSuccessStyles`, `withTime`, `withWeekNumbers`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### MonthPicker — D:MonthPicker

来源：[@mantine/dates](https://mantine.dev/dates/month-picker/)；对应 `P9-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getMonthControlProps`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthsListFormat`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthSelect`, `onNextDecade`, `onNextYear`, `onPreviousDecade`, `onPreviousYear`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`

**官方功能章节：** Usage；Allow deselect；Multiple dates；Dates range；Single date in range；Presets；Default date；Controlled date；Min and max date；Add props to year and month control；Number of columns；Max level；Full width；Size；Change year and months controls format；Change label format；Localization；Native level select；Accessibility

**本地实现：** [src/period_model.rs](../src/period_model.rs)、[src/period_picker.rs](../src/period_picker.rs)、[src/period_picker_input.rs](../src/period_picker_input.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `checked_add`, `clamp`, `class`, `close_label`, `contains`, `default_open`, `default_value`, `default_year`, `direction`, `enabled`, `end`, `exclude_value`, `exclude_values`, `field`, `focus`, `form_name`, `format_iso`, `from_dates`, `grid_ref`, `insert`, `invalid`, `is_complete`, `iter`, `len_periods`, `max`, `min`, `mode`, `month_picker`, `month_picker_input`, `name`, `new`, `node_ref`, `on`, `on_open_change`, `on_reject`, `on_value_change`, `on_year_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `start`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`, `year`, `year_picker`, `year_picker_input`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getMonthControlProps`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthsListFormat`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onMonthSelect`, `onNextDecade`, `onNextYear`, `onPreviousDecade`, `onPreviousYear`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`

### MonthPickerInput — D:MonthPickerInput

来源：[@mantine/dates](https://mantine.dev/dates/month-picker-input/)；对应 `P9-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getMonthControlProps`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthsListFormat`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onNextDecade`, `onNextYear`, `onPreviousDecade`, `onPreviousYear`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`

**官方功能章节：** MonthPicker props；Usage；Multiple dates；Dates range；Presets；Open picker in modal；Value format；Value formatter；Clearable；Disabled state；Min and max dates；Control props；Label format；List format；Max level；Input props；With icon；Native level select；Accessibility

**本地实现：** [src/period_model.rs](../src/period_model.rs)、[src/period_picker.rs](../src/period_picker.rs)、[src/period_picker_input.rs](../src/period_picker_input.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `checked_add`, `clamp`, `class`, `close_label`, `contains`, `default_open`, `default_value`, `default_year`, `direction`, `enabled`, `end`, `exclude_value`, `exclude_values`, `field`, `focus`, `form_name`, `format_iso`, `from_dates`, `grid_ref`, `insert`, `invalid`, `is_complete`, `iter`, `len_periods`, `max`, `min`, `mode`, `month_picker`, `month_picker_input`, `name`, `new`, `node_ref`, `on`, `on_open_change`, `on_reject`, `on_value_change`, `on_year_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `start`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`, `year`, `year_picker`, `year_picker_input`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getMonthControlProps`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `level`, `loading`, `loadingPosition`, `locale`, `maxDate`, `maxLevel`, `minDate`, `modalProps`, `monthsListFormat`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onLevelChange`, `onNextDecade`, `onNextYear`, `onPreviousDecade`, `onPreviousYear`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearLabelFormat`, `yearsListFormat`

### YearPicker — D:YearPicker

来源：[@mantine/dates](https://mantine.dev/dates/year-picker/)；对应 `P9-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `locale`, `maxDate`, `minDate`, `modalProps`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onNextDecade`, `onPreviousDecade`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearsListFormat`

**官方功能章节：** Usage；Allow deselect；Multiple dates；Dates range；Single date in range；Presets；Default date；Controlled date；Min and max date；Add props to year control；Number of columns；Full width；Size；Change year controls format；Change decade label format；Accessibility；CHARTS COMPONENTS AND FEATURES

**本地实现：** [src/period_model.rs](../src/period_model.rs)、[src/period_picker.rs](../src/period_picker.rs)、[src/period_picker_input.rs](../src/period_picker_input.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `checked_add`, `clamp`, `class`, `close_label`, `contains`, `default_open`, `default_value`, `default_year`, `direction`, `enabled`, `end`, `exclude_value`, `exclude_values`, `field`, `focus`, `form_name`, `format_iso`, `from_dates`, `grid_ref`, `insert`, `invalid`, `is_complete`, `iter`, `len_periods`, `max`, `min`, `mode`, `month_picker`, `month_picker_input`, `name`, `new`, `node_ref`, `on`, `on_open_change`, `on_reject`, `on_value_change`, `on_year_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `start`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`, `year`, `year_picker`, `year_picker_input`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `locale`, `maxDate`, `minDate`, `modalProps`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onNextDecade`, `onPreviousDecade`, `onYearSelect`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearsListFormat`

### YearPickerInput — D:YearPickerInput

来源：[@mantine/dates](https://mantine.dev/dates/year-picker-input/)；对应 `P9-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `locale`, `maxDate`, `minDate`, `modalProps`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onNextDecade`, `onPreviousDecade`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `sortDates`, `success`, `successProps`, `type`, `value`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearsListFormat`

**官方功能章节：** YearPicker props；Usage；Multiple dates；Dates range；Presets；Open picker in modal；Value format；Value formatter；Clearable；Disabled state；Min and max dates；Control props；Year label format；Decade label format；Input props；With icon；Accessibility

**本地实现：** [src/period_model.rs](../src/period_model.rs)、[src/period_picker.rs](../src/period_picker.rs)、[src/period_picker_input.rs](../src/period_picker_input.rs)

**本地公开入口（同族）：** `activate`, `allow_deselect`, `anchor`, `checked_add`, `clamp`, `class`, `close_label`, `contains`, `default_open`, `default_value`, `default_year`, `direction`, `enabled`, `end`, `exclude_value`, `exclude_values`, `field`, `focus`, `form_name`, `format_iso`, `from_dates`, `grid_ref`, `insert`, `invalid`, `is_complete`, `iter`, `len_periods`, `max`, `min`, `mode`, `month_picker`, `month_picker_input`, `name`, `new`, `node_ref`, `on`, `on_open_change`, `on_reject`, `on_value_change`, `on_year_change`, `open`, `panel_ref`, `parse_iso`, `readonly`, `remove`, `required`, `size`, `start`, `style`, `theme`, `title`, `trigger_label`, `trigger_ref`, `try_from_iter`, `unstyled`, `validate`, `value`, `year`, `year_picker`, `year_picker_input`

**需要语义映射的属性候选：** `allowSingleDateInRange`, `ariaLabels`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeOnChange`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `description`, `descriptionProps`, `disabled`, `dropdownType`, `error`, `errorProps`, `fullWidth`, `getYearControlProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `labelSeparator`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `locale`, `maxDate`, `minDate`, `modalProps`, `nextLabel`, `numberOfColumns`, `onChange`, `onDateChange`, `onDropdownClose`, `onNextDecade`, `onPreviousDecade`, `placeholder`, `pointer`, `popoverProps`, `presets`, `previousLabel`, `radius`, `readOnly`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `sortDates`, `success`, `successProps`, `type`, `valueFormat`, `valueFormatter`, `withAsterisk`, `withCellSpacing`, `withErrorStyles`, `withNativeLevelSelect`, `withSuccessStyles`, `wrapperProps`, `yearsListFormat`

### TimeInput — D:TimeInput

来源：[@mantine/dates](https://mantine.dev/dates/time-input/)；对应 `P9-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxTime`, `minTime`, `pointer`, `radius`, `required`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `size`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSeconds`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；TimePicker component；Controlled；Show browser picker；With seconds；With icon；Disabled state；Accessibility

**本地实现：** [src/time_input.rs](../src/time_input.rs)、[src/time_model.rs](../src/time_model.rs)、[src/time_value.rs](../src/time_value.rs)

**本地公开入口（同族）：** `bounds`, `clamp`, `class`, `classify_time_draft`, `contains`, `default_value`, `empty_label`, `enabled`, `field`, `focus`, `form_name`, `format`, `invalid`, `is_error`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_value_change`, `parse`, `proposal`, `readonly`, `required`, `size`, `style`, `theme`, `time_input`, `time_value`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `maxTime`, `minTime`, `pointer`, `radius`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `success`, `successProps`, `withAsterisk`, `withErrorStyles`, `withSeconds`, `withSuccessStyles`, `wrapperProps`

### TimePicker — D:TimePicker

来源：[@mantine/dates](https://mantine.dev/dates/time-picker/)；对应 `P9-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `amPmInputLabel`, `amPmLabels`, `amPmRef`, `amPmSelectProps`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeDropdownOnPresetSelect`, `defaultValue`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `form`, `format`, `hiddenInputProps`, `hoursInputLabel`, `hoursInputProps`, `hoursPlaceholder`, `hoursRef`, `hoursStep`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `max`, `maxDropdownContentHeight`, `min`, `minHoursDigits`, `minutesInputLabel`, `minutesInputProps`, `minutesPlaceholder`, `minutesRef`, `minutesStep`, `name`, `onBlur`, `onChange`, `onFocus`, `pasteSplit`, `pointer`, `popoverProps`, `presets`, `radius`, `readOnly`, `required`, `reverseTimeControlsList`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `secondsInputLabel`, `secondsInputProps`, `secondsPlaceholder`, `secondsRef`, `secondsStep`, `size`, `success`, `successProps`, `type`, `value`, `withAsterisk`, `withDropdown`, `withErrorStyles`, `withSeconds`, `withSuccessStyles`, `wrapperProps`

**官方功能章节：** Usage；Controlled；With seconds；Duration type；Min hours digits；12-hour format；Change am/pm labels；Min and max values；With dropdown；Hours/minutes/seconds step；Control dropdown opened state；Time presets；Time presets groups；Time presets range；Close dropdown on preset select；Dropdown position；Dropdown width；Paste events；Clearable；Disabled；Read only；Input props；Get refs of inner inputs；onFocus and onBlur events；Accessibility

**本地实现：** [src/time_grid.rs](../src/time_grid.rs)、[src/time_grid_model.rs](../src/time_grid_model.rs)、[src/time_parts.rs](../src/time_parts.rs)、[src/time_picker.rs](../src/time_picker.rs)

**本地公开入口（同族）：** `at`, `bounds`, `class`, `classify`, `default_value`, `direction`, `disable_time`, `disable_times`, `enabled`, `field`, `focus`, `form_name`, `format`, `from_time`, `grid`, `hour_ref`, `hour_step`, `index_of`, `invalid`, `is_error`, `minute_ref`, `minute_step`, `name`, `nearest_index`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_value_change`, `proposal`, `readonly`, `required`, `rows`, `second_ref`, `second_step`, `size`, `spec`, `style`, `theme`, `time_grid`, `time_picker`, `unstyled`, `value`

**需要语义映射的属性候选：** `amPmInputLabel`, `amPmLabels`, `amPmRef`, `amPmSelectProps`, `clearButtonProps`, `clearSectionMode`, `clearable`, `closeDropdownOnPresetSelect`, `description`, `descriptionProps`, `disabled`, `error`, `errorProps`, `form`, `hiddenInputProps`, `hoursInputLabel`, `hoursInputProps`, `hoursPlaceholder`, `hoursRef`, `hoursStep`, `inputContainer`, `inputSize`, `inputWrapperOrder`, `label`, `labelProps`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `loading`, `loadingPosition`, `max`, `maxDropdownContentHeight`, `min`, `minHoursDigits`, `minutesInputLabel`, `minutesInputProps`, `minutesPlaceholder`, `minutesRef`, `minutesStep`, `onBlur`, `onChange`, `onFocus`, `pasteSplit`, `pointer`, `popoverProps`, `presets`, `radius`, `readOnly`, `reverseTimeControlsList`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `scrollAreaProps`, `secondsInputLabel`, `secondsInputProps`, `secondsPlaceholder`, `secondsRef`, `secondsStep`, `success`, `successProps`, `type`, `withAsterisk`, `withDropdown`, `withErrorStyles`, `withSeconds`, `withSuccessStyles`, `wrapperProps`

### TimeGrid — D:TimeGrid

来源：[@mantine/dates](https://mantine.dev/dates/time-grid/)；对应 `P9-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDeselect`, `amPmLabels`, `data`, `defaultValue`, `disableTime`, `disabled`, `format`, `getControlProps`, `maxTime`, `minTime`, `onChange`, `radius`, `simpleGridProps`, `size`, `value`, `withSeconds`

**官方功能章节：** Usage；Controlled；data prop；Min and max time；Disable specific controls；Allow deselect；Change AM/PM labels；Disabled

**本地实现：** [src/time_grid.rs](../src/time_grid.rs)、[src/time_grid_model.rs](../src/time_grid_model.rs)、[src/time_parts.rs](../src/time_parts.rs)、[src/time_picker.rs](../src/time_picker.rs)

**本地公开入口（同族）：** `at`, `bounds`, `class`, `classify`, `default_value`, `direction`, `disable_time`, `disable_times`, `enabled`, `field`, `focus`, `form_name`, `format`, `from_time`, `grid`, `hour_ref`, `hour_step`, `index_of`, `invalid`, `is_error`, `minute_ref`, `minute_step`, `name`, `nearest_index`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_value_change`, `proposal`, `readonly`, `required`, `rows`, `second_ref`, `second_step`, `size`, `spec`, `style`, `theme`, `time_grid`, `time_picker`, `unstyled`, `value`

**需要语义映射的属性候选：** `allowDeselect`, `amPmLabels`, `data`, `disabled`, `getControlProps`, `maxTime`, `minTime`, `onChange`, `radius`, `simpleGridProps`, `withSeconds`

### TimeValue — D:TimeValue

来源：[@mantine/dates](https://mantine.dev/dates/time-value/)；对应 `P9-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `amPmLabels`, `format`, `value`, `withSeconds`

**官方功能章节：** Usage；With seconds；AM/PM labels；Date object

**本地实现：** [src/time_input.rs](../src/time_input.rs)、[src/time_model.rs](../src/time_model.rs)、[src/time_value.rs](../src/time_value.rs)

**本地公开入口（同族）：** `bounds`, `clamp`, `class`, `classify_time_draft`, `contains`, `default_value`, `empty_label`, `enabled`, `field`, `focus`, `form_name`, `format`, `invalid`, `is_error`, `name`, `new`, `node_ref`, `on`, `on_commit`, `on_draft_change`, `on_reject`, `on_value_change`, `parse`, `proposal`, `readonly`, `required`, `size`, `style`, `theme`, `time_input`, `time_value`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `amPmLabels`, `withSeconds`

### MiniCalendar — D:MiniCalendar

来源：[@mantine/dates](https://mantine.dev/dates/mini-calendar/)；对应 `P9-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `date`, `defaultDate`, `getDayProps`, `locale`, `maxDate`, `minDate`, `monthLabelFormat`, `nextControlProps`, `numberOfDays`, `onChange`, `onDateChange`, `onNext`, `onPrevious`, `previousControlProps`, `size`, `value`

**官方功能章节：** Usage；Number of days；getDayProps；Min and max dates；Localization；Accessibility

**本地实现：** [../glory/crates/render/src/grid.rs](../../glory/crates/render/src/grid.rs)、[src/calendar.rs](../src/calendar.rs)、[src/dates.rs](../src/dates.rs)

**本地公开入口（同族）：** `add_months`, `allow_deselect`, `calendar`, `cells`, `checked_add`, `checked_add_days`, `clamp`, `class`, `contains`, `date_label`, `days`, `days_until`, `default_month`, `default_value`, `direction`, `disable_weekday`, `enabled`, `exclude_date`, `exclude_dates`, `first`, `focus`, `from_ordinal`, `index_of`, `key`, `last`, `len_days`, `mini_calendar`, `month`, `month_allowed`, `month_label`, `month_name`, `name`, `new`, `node_ref`, `on`, `on_month_change`, `on_value_change`, `ordinal`, `parse_iso`, `readonly`, `resolve_fixed`, `seconds_from_midnight`, `size`, `style`, `theme`, `to_local`, `unstyled`, `value`, `weekday`, `weekday_long`, `weekday_short`, `with_nanosecond`

**需要语义映射的属性候选：** `date`, `defaultDate`, `getDayProps`, `locale`, `maxDate`, `minDate`, `monthLabelFormat`, `nextControlProps`, `numberOfDays`, `onChange`, `onDateChange`, `onNext`, `onPrevious`, `previousControlProps`

### Calendar — D:Calendar

来源：[@mantine/dates](https://mantine.dev/dates/calendar/)；对应 `P9-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `ariaLabels`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `enableKeyboardNavigation`, `excludeDate`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `hasNextLevel`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `label`, `level`, `levelControlAriaLabel`, `locale`, `maxDate`, `maxLevel`, `minDate`, `minLevel`, `monthLabelFormat`, `monthsListFormat`, `nextDisabled`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onDateChange`, `onLevelChange`, `onLevelClick`, `onMonthMouseEnter`, `onMonthSelect`, `onNext`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPrevious`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `previousDisabled`, `previousIcon`, `previousLabel`, `renderDay`, `size`, `static`, `weekdayFormat`, `weekendDays`, `withCellSpacing`, `withNativeLevelSelect`, `withNext`, `withPrevious`, `withWeekNumbers`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

**官方功能章节：** Usage；Custom date pickers；Full width；Static prop；Native level select

**本地实现：** [../glory/crates/render/src/grid.rs](../../glory/crates/render/src/grid.rs)、[src/calendar.rs](../src/calendar.rs)、[src/dates.rs](../src/dates.rs)

**本地公开入口（同族）：** `add_months`, `allow_deselect`, `calendar`, `cells`, `checked_add`, `checked_add_days`, `clamp`, `class`, `contains`, `date_label`, `days`, `days_until`, `default_month`, `default_value`, `direction`, `disable_weekday`, `enabled`, `exclude_date`, `exclude_dates`, `first`, `focus`, `from_ordinal`, `index_of`, `key`, `last`, `len_days`, `mini_calendar`, `month`, `month_allowed`, `month_label`, `month_name`, `name`, `new`, `node_ref`, `on`, `on_month_change`, `on_value_change`, `ordinal`, `parse_iso`, `readonly`, `resolve_fixed`, `seconds_from_midnight`, `size`, `style`, `theme`, `to_local`, `unstyled`, `value`, `weekday`, `weekday_long`, `weekday_short`, `with_nanosecond`

**需要语义映射的属性候选：** `ariaLabels`, `columnsToScroll`, `date`, `decadeLabelFormat`, `defaultDate`, `defaultLevel`, `enableKeyboardNavigation`, `firstDayOfWeek`, `fullWidth`, `getDayAriaLabel`, `getDayProps`, `getMonthControlProps`, `getYearControlProps`, `hasNextLevel`, `headerControlsOrder`, `hideOutsideDates`, `hideWeekdays`, `highlightToday`, `label`, `level`, `levelControlAriaLabel`, `locale`, `maxDate`, `maxLevel`, `minDate`, `minLevel`, `monthLabelFormat`, `monthsListFormat`, `nextDisabled`, `nextIcon`, `nextLabel`, `numberOfColumns`, `onDateChange`, `onLevelChange`, `onLevelClick`, `onMonthMouseEnter`, `onMonthSelect`, `onNext`, `onNextDecade`, `onNextMonth`, `onNextYear`, `onPrevious`, `onPreviousDecade`, `onPreviousMonth`, `onPreviousYear`, `onYearMouseEnter`, `onYearSelect`, `previousDisabled`, `previousIcon`, `previousLabel`, `renderDay`, `static`, `weekdayFormat`, `weekendDays`, `withCellSpacing`, `withNativeLevelSelect`, `withNext`, `withPrevious`, `withWeekNumbers`, `yearLabelFormat`, `yearsListFormat`, `yearsSelectRange`

### AreaChart — C:AreaChart

来源：[@mantine/charts](https://mantine.dev/charts/area-chart/)；对应 `P11-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `activeDotProps`, `areaChartProps`, `areaProps`, `brushProps`, `children`, `connectNulls`, `curveType`, `data`, `dataKey`, `dotProps`, `fillOpacity`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `splitColors`, `splitOffset`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `withBrush`, `withDots`, `withGradient`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Stacked area chart；Percent area chart；Streamgraph；Split area chart；Split colors；Legend；Legend position；Series labels；Set curve type per line；Connect nulls；Points labels；X and Y axis props；Axis labels；X axis offset；Y axis scale；Right Y axis；Rotate x-axis labels；Value formatter；Area color；Change area color depending on color scheme；Stroke dash array；Grid and text colors；Tooltip animation；Units；Custom tooltip；Remove tooltip；Customize dots；Stroke width；Fill opacity；Sync multiple AreaCharts；Vertical orientation；Dashed area line；Reference lines；Reference areas；Reference dots；Brush；Brush props；ChartBrush component

**本地实现：** [src/chart_model.rs](../src/chart_model.rs)、[src/chart_scene.rs](../src/chart_scene.rs)、[src/chart_widget.rs](../src/chart_widget.rs)

**本地公开入口（同族）：** `area_chart`, `bar_chart`, `build`, `chart`, `chart_theme`, `class`, `composite_chart`, `data`, `data_table`, `default_options`, `domain`, `enabled`, `focus`, `fraction`, `frame`, `from_theme`, `kind`, `legend`, `line_chart`, `max`, `message`, `min`, `name`, `nearest`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `pointer_selection`, `selection`, `size`, `style`, `text`, `theme`, `ticks`, `to_svg`, `unstyled`, `validate`

**需要语义映射的属性候选：** `accessibilityLayer`, `activeDotProps`, `areaChartProps`, `areaProps`, `brushProps`, `children`, `connectNulls`, `curveType`, `dataKey`, `dotProps`, `fillOpacity`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `splitColors`, `splitOffset`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `withBrush`, `withDots`, `withGradient`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### BarChart — C:BarChart

来源：[@mantine/charts](https://mantine.dev/charts/bar-chart/)；对应 `P11-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `barChartProps`, `barLabelColor`, `barProps`, `brushProps`, `children`, `cursorFill`, `data`, `dataKey`, `fillOpacity`, `getBarColor`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `maxBarWidth`, `minBarSize`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `valueLabelProps`, `withBarValueLabel`, `withBrush`, `withLegend`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Stacked bar chart；Mixed stacked bar chart；Percent bar chart；Waterfall bar chart；SVG pattern as bar fill；Bar color based on value；Legend；Legend position；Series labels；X and Y axis props；Axis labels；X axis offset；Y axis scale；Value formatter；Area color；Bar props；Minimum bar size；Change bar color depending on color scheme；Stroke dash array；Grid and text colors；Tooltip animation；Units；Custom tooltip；Remove tooltip；Sync multiple BarCharts；Vertical orientation；Reference lines；Reference areas；Reference dots；Bar value label；Bar value label props；Bar overlays；Brush

**本地实现：** [src/chart_model.rs](../src/chart_model.rs)、[src/chart_scene.rs](../src/chart_scene.rs)、[src/chart_widget.rs](../src/chart_widget.rs)

**本地公开入口（同族）：** `area_chart`, `bar_chart`, `build`, `chart`, `chart_theme`, `class`, `composite_chart`, `data`, `data_table`, `default_options`, `domain`, `enabled`, `focus`, `fraction`, `frame`, `from_theme`, `kind`, `legend`, `line_chart`, `max`, `message`, `min`, `name`, `nearest`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `pointer_selection`, `selection`, `size`, `style`, `text`, `theme`, `ticks`, `to_svg`, `unstyled`, `validate`

**需要语义映射的属性候选：** `accessibilityLayer`, `barChartProps`, `barLabelColor`, `barProps`, `brushProps`, `children`, `cursorFill`, `dataKey`, `fillOpacity`, `getBarColor`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `maxBarWidth`, `minBarSize`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `valueLabelProps`, `withBarValueLabel`, `withBrush`, `withLegend`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### LineChart — C:LineChart

来源：[@mantine/charts](https://mantine.dev/charts/line-chart/)；对应 `P11-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `activeDotProps`, `brushProps`, `children`, `connectNulls`, `curveType`, `data`, `dataKey`, `dotProps`, `fillOpacity`, `gradientStops`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `lineChartProps`, `lineProps`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `withBrush`, `withDots`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Gradient type；Legend；Legend position；Series labels；Connect nulls；Points labels；X and Y axis props；Axis labels；X axis offset；Y axis scale；Right Y axis；Value formatter；Set curve type per line；Line color；Change line color depending on color scheme；Stroke dash array；Grid and text colors；Tooltip animation；Units；Custom tooltip；Remove tooltip；Customize dots；Stroke width；Sync multiple LineCharts；Vertical orientation；Dashed line；Reference lines；Reference areas；Reference dots；Brush

**本地实现：** [src/chart_model.rs](../src/chart_model.rs)、[src/chart_scene.rs](../src/chart_scene.rs)、[src/chart_widget.rs](../src/chart_widget.rs)

**本地公开入口（同族）：** `area_chart`, `bar_chart`, `build`, `chart`, `chart_theme`, `class`, `composite_chart`, `data`, `data_table`, `default_options`, `domain`, `enabled`, `focus`, `fraction`, `frame`, `from_theme`, `kind`, `legend`, `line_chart`, `max`, `message`, `min`, `name`, `nearest`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `pointer_selection`, `selection`, `size`, `style`, `text`, `theme`, `ticks`, `to_svg`, `unstyled`, `validate`

**需要语义映射的属性候选：** `accessibilityLayer`, `activeDotProps`, `brushProps`, `children`, `connectNulls`, `curveType`, `dataKey`, `dotProps`, `fillOpacity`, `gradientStops`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `lineChartProps`, `lineProps`, `orientation`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `type`, `unit`, `valueFormatter`, `withBrush`, `withDots`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### CompositeChart — C:CompositeChart

来源：[@mantine/charts](https://mantine.dev/charts/composite-chart/)；对应 `P11-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `activeDotProps`, `areaProps`, `barProps`, `brushProps`, `children`, `composedChartProps`, `connectNulls`, `curveType`, `data`, `dataKey`, `dotProps`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `lineProps`, `maxBarWidth`, `minBarSize`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `valueFormatter`, `withBarValueLabel`, `withBrush`, `withDots`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Legend；Legend position；Series labels；Points labels；X and Y axis props；Axis labels；X axis offset；Y axis scale；Right Y axis；Rotate x-axis labels；Value formatter；Chart color；Change chart color depending on color scheme；Stroke dash array；Grid and text colors；Tooltip animation；Units；Custom tooltip；Remove tooltip；Customize dots；Stroke width；Sync multiple charts；Dashed lines；Reference lines；Reference areas；Reference dots；Brush

**本地实现：** [src/chart_model.rs](../src/chart_model.rs)、[src/chart_scene.rs](../src/chart_scene.rs)、[src/chart_widget.rs](../src/chart_widget.rs)

**本地公开入口（同族）：** `area_chart`, `bar_chart`, `build`, `chart`, `chart_theme`, `class`, `composite_chart`, `data`, `data_table`, `default_options`, `domain`, `enabled`, `focus`, `fraction`, `frame`, `from_theme`, `kind`, `legend`, `line_chart`, `max`, `message`, `min`, `name`, `nearest`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `pointer_selection`, `selection`, `size`, `style`, `text`, `theme`, `ticks`, `to_svg`, `unstyled`, `validate`

**需要语义映射的属性候选：** `accessibilityLayer`, `activeDotProps`, `areaProps`, `barProps`, `brushProps`, `children`, `composedChartProps`, `connectNulls`, `curveType`, `dataKey`, `dotProps`, `gridAxis`, `gridColor`, `gridProps`, `legendProps`, `lineProps`, `maxBarWidth`, `minBarSize`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `series`, `strokeDasharray`, `strokeWidth`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `valueFormatter`, `withBarValueLabel`, `withBrush`, `withDots`, `withLegend`, `withPointLabels`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### CandlestickChart — C:CandlestickChart

来源：[@mantine/charts](https://mantine.dev/charts/candlestick-chart/)；对应 `P11-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `candleStrokeWidth`, `children`, `composedChartProps`, `data`, `dataKey`, `dataKeys`, `downColor`, `gridAxis`, `gridColor`, `gridProps`, `labels`, `maxCandleWidth`, `referenceAreas`, `referenceDots`, `referenceLines`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `upColor`, `valueFormatter`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Data；Custom data keys；Colors；Candle width；Value formatter；Tooltip labels；Axis labels；Reference lines

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `candleStrokeWidth`, `children`, `composedChartProps`, `dataKey`, `dataKeys`, `downColor`, `gridAxis`, `gridColor`, `gridProps`, `labels`, `maxCandleWidth`, `referenceAreas`, `referenceDots`, `referenceLines`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `upColor`, `valueFormatter`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### DonutChart — C:DonutChart

来源：[@mantine/charts](https://mantine.dev/charts/donut-chart/)；对应 `P11-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `cellProps`, `chartLabel`, `children`, `data`, `endAngle`, `labelColor`, `labelsType`, `legendProps`, `paddingAngle`, `pieChartProps`, `pieProps`, `size`, `startAngle`, `strokeColor`, `strokeWidth`, `thickness`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLabelsLine`, `withLegend`, `withTooltip`

**官方功能章节：** Usage；Legend；Segments labels；Size and thickness；Padding angle；Segment color；Tooltip data source；Without tooltip；Chart label；Start and end angle；Segments stroke

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `cellProps`, `chartLabel`, `children`, `endAngle`, `labelColor`, `labelsType`, `legendProps`, `paddingAngle`, `pieChartProps`, `pieProps`, `startAngle`, `strokeColor`, `strokeWidth`, `thickness`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLabelsLine`, `withLegend`, `withTooltip`

### PieChart — C:PieChart

来源：[@mantine/charts](https://mantine.dev/charts/pie-chart/)；对应 `P11-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `cellProps`, `children`, `data`, `endAngle`, `labelColor`, `labelsPosition`, `labelsType`, `legendProps`, `paddingAngle`, `pieChartProps`, `pieProps`, `size`, `startAngle`, `strokeColor`, `strokeWidth`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLabelsLine`, `withLegend`, `withTooltip`

**官方功能章节：** Usage；Legend；Segments labels；Size；Segment color；Enable tooltip；Tooltip data source；Start and end angle；Segments stroke

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `cellProps`, `children`, `endAngle`, `labelColor`, `labelsPosition`, `labelsType`, `legendProps`, `paddingAngle`, `pieChartProps`, `pieProps`, `startAngle`, `strokeColor`, `strokeWidth`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLabelsLine`, `withLegend`, `withTooltip`

### FunnelChart — C:FunnelChart

来源：[@mantine/charts](https://mantine.dev/charts/funnel-chart/)；对应 `P11-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `children`, `data`, `funnelChartProps`, `funnelProps`, `labelColor`, `labelsPosition`, `legendProps`, `size`, `strokeColor`, `strokeWidth`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLegend`, `withTooltip`

**官方功能章节：** Usage；Legend；Segments labels；Size and thickness；Segment color；Tooltip data source；Without tooltip；Segments stroke

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `children`, `funnelChartProps`, `funnelProps`, `labelColor`, `labelsPosition`, `legendProps`, `strokeColor`, `strokeWidth`, `tooltipAnimationDuration`, `tooltipDataSource`, `tooltipProps`, `valueFormatter`, `withLabels`, `withLegend`, `withTooltip`

### RadarChart — C:RadarChart

来源：[@mantine/charts](https://mantine.dev/charts/radar-chart/)；对应 `P11-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `activeDotProps`, `children`, `data`, `dataKey`, `dotProps`, `gridColor`, `legendProps`, `polarAngleAxisProps`, `polarGridProps`, `polarRadiusAxisProps`, `radarChartProps`, `radarProps`, `series`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `withDots`, `withLegend`, `withPolarAngleAxis`, `withPolarGrid`, `withPolarRadiusAxis`, `withTooltip`

**官方功能章节：** Usage；Multiple series；Change color；Hide/show chart parts；With tooltip and dots；Recharts props；Legend

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `activeDotProps`, `children`, `dataKey`, `dotProps`, `gridColor`, `legendProps`, `polarAngleAxisProps`, `polarGridProps`, `polarRadiusAxisProps`, `radarChartProps`, `radarProps`, `series`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `withDots`, `withLegend`, `withPolarAngleAxis`, `withPolarGrid`, `withPolarRadiusAxis`, `withTooltip`

### ScatterChart — C:ScatterChart

来源：[@mantine/charts](https://mantine.dev/charts/scatter-chart/)；对应 `P11-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `data`, `dataKey`, `gridAxis`, `gridColor`, `gridProps`, `labels`, `legendProps`, `orientation`, `pointLabels`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `scatterChartProps`, `scatterProps`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `valueFormatter`, `withLegend`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

**官方功能章节：** Usage；Multiple series；Legend；Legend position；X and Y axis props；Right Y axis；Value formatter；Point labels；Grid and text colors；Stroke dash array；Units；Tooltip labels；Custom tooltip；Remove tooltip；Tooltip animation；Customize dots；Reference lines；Reference areas；Reference dots

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `dataKey`, `gridAxis`, `gridColor`, `gridProps`, `labels`, `legendProps`, `orientation`, `pointLabels`, `referenceAreas`, `referenceDots`, `referenceLines`, `rightYAxisLabel`, `rightYAxisProps`, `scatterChartProps`, `scatterProps`, `strokeDasharray`, `textColor`, `tickLine`, `tooltipAnimationDuration`, `tooltipProps`, `unit`, `valueFormatter`, `withLegend`, `withRightYAxis`, `withTooltip`, `withXAxis`, `withYAxis`, `xAxisLabel`, `xAxisProps`, `yAxisLabel`, `yAxisProps`

### BubbleChart — C:BubbleChart

来源：[@mantine/charts](https://mantine.dev/charts/bubble-chart/)；对应 `P11-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `color`, `data`, `dataKey`, `gridColor`, `label`, `range`, `scatterProps`, `textColor`, `tooltipProps`, `valueFormatter`, `withTooltip`, `xAxisProps`, `yAxisProps`, `zAxisProps`

**官方功能章节：** Usage；Change color；Change area color depending on color scheme；Remove tooltip；Value formatter；Grid and text colors

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `color`, `dataKey`, `gridColor`, `label`, `range`, `scatterProps`, `textColor`, `tooltipProps`, `valueFormatter`, `withTooltip`, `xAxisProps`, `yAxisProps`, `zAxisProps`

### RadialBarChart — C:RadialBarChart

来源：[@mantine/charts](https://mantine.dev/charts/radial-bar-chart/)；对应 `P11-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accessibilityLayer`, `barSize`, `data`, `dataKey`, `emptyBackgroundColor`, `endAngle`, `legendProps`, `radialBarChartProps`, `radialBarProps`, `startAngle`, `tooltipProps`, `withBackground`, `withLabels`, `withLegend`, `withTooltip`

**官方功能章节：** Usage；Change color；Legend；Labels；Hide tooltip

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `accessibilityLayer`, `barSize`, `dataKey`, `emptyBackgroundColor`, `endAngle`, `legendProps`, `radialBarChartProps`, `radialBarProps`, `startAngle`, `tooltipProps`, `withBackground`, `withLabels`, `withLegend`, `withTooltip`

### Sparkline — C:Sparkline

来源：[@mantine/charts](https://mantine.dev/charts/sparkline/)；对应 `P11-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `areaProps`, `color`, `connectNulls`, `curveType`, `data`, `fillOpacity`, `strokeWidth`, `trendColors`, `withGradient`

**官方功能章节：** Usage；Change area color depending on color scheme；Trend colors

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `areaProps`, `color`, `connectNulls`, `curveType`, `fillOpacity`, `strokeWidth`, `trendColors`, `withGradient`

### Heatmap — C:Heatmap

来源：[@mantine/charts](https://mantine.dev/charts/heatmap/)；对应 `P11-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `colors`, `data`, `domain`, `endDate`, `firstDayOfWeek`, `fontSize`, `gap`, `getRectProps`, `getTooltipLabel`, `legendLabels`, `monthLabels`, `monthLabelsPosition`, `monthsLabelsHeight`, `rectRadius`, `rectSize`, `splitMonths`, `startDate`, `tooltipProps`, `weekdayLabels`, `weekdaysLabelsWidth`, `withLegend`, `withMonthLabels`, `withOutsideDates`, `withTooltip`, `withWeekdayLabels`

**官方功能章节：** Usage；Data format；With tooltip；Change colors；Colors depending on color scheme；Values domain；Weekdays and months labels；Month labels position；Change labels text；Rect size, gap and radius；Pass props to rect；Hide outside dates；First day of week；Legend；Split months

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `colors`, `domain`, `endDate`, `firstDayOfWeek`, `fontSize`, `gap`, `getRectProps`, `getTooltipLabel`, `legendLabels`, `monthLabels`, `monthLabelsPosition`, `monthsLabelsHeight`, `rectRadius`, `rectSize`, `splitMonths`, `startDate`, `tooltipProps`, `weekdayLabels`, `weekdaysLabelsWidth`, `withLegend`, `withMonthLabels`, `withOutsideDates`, `withTooltip`, `withWeekdayLabels`

### BarsList — C:BarsList

来源：[@mantine/charts](https://mantine.dev/charts/bars-list/)；对应 `P11-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `autoContrast`, `barColor`, `barGap`, `barHeight`, `barTextColor`, `barsLabel`, `data`, `getBarProps`, `minBarSize`, `renderBar`, `valueFormatter`, `valueLabel`

**官方功能章节：** Usage；Data format；Value formatter；Labels；Bar gap；Minimum bar size；Bar height；Bar color；Auto contrast；Custom colors；Get bar props；Custom bar rendering

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `autoContrast`, `barColor`, `barGap`, `barHeight`, `barTextColor`, `barsLabel`, `getBarProps`, `minBarSize`, `renderBar`, `valueFormatter`, `valueLabel`

### BulletChart — C:BulletChart

来源：[@mantine/charts](https://mantine.dev/charts/bullet-chart/)；对应 `P11-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `barColor`, `barSize`, `getTooltipLabel`, `label`, `orientation`, `ranges`, `size`, `target`, `targetColor`, `targetRatio`, `targetSize`, `value`, `valueFormatter`, `withTooltip`

**官方功能章节：** Usage；Data format；Label；Vertical orientation；Bar color；Chart size；Value formatter；Without target

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `barColor`, `barSize`, `getTooltipLabel`, `label`, `orientation`, `ranges`, `target`, `targetColor`, `targetRatio`, `targetSize`, `value`, `valueFormatter`, `withTooltip`

### GaugeChart — C:GaugeChart

来源：[@mantine/charts](https://mantine.dev/charts/gauge-chart/)；对应 `P11-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `endAngle`, `filledColor`, `label`, `max`, `min`, `roundCaps`, `sections`, `size`, `startAngle`, `target`, `targetColor`, `targetSize`, `thickness`, `trackColor`, `value`, `valueFormatter`

**官方功能章节：** Usage；Sections；Target marker；Start and end angle；Thickness；Custom label；Round caps；Accessibility

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `endAngle`, `filledColor`, `label`, `max`, `min`, `roundCaps`, `sections`, `startAngle`, `target`, `targetColor`, `targetSize`, `thickness`, `trackColor`, `value`, `valueFormatter`

### WaffleChart — C:WaffleChart

来源：[@mantine/charts](https://mantine.dev/charts/waffle-chart/)；对应 `P11-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `cellRadius`, `columns`, `data`, `emptyColor`, `fillDirection`, `gap`, `getTooltipLabel`, `legendPosition`, `rows`, `size`, `total`, `withLegend`, `withTooltip`

**官方功能章节：** Usage；Total；Tooltip；Legend；Gap and grid size；Fill direction；Empty cells color；Accessibility；CODE-HIGHLIGHT COMPONENTS AND FEATURES

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `cellRadius`, `columns`, `emptyColor`, `fillDirection`, `gap`, `getTooltipLabel`, `legendPosition`, `rows`, `total`, `withLegend`, `withTooltip`

### Treemap — C:Treemap

来源：[@mantine/charts](https://mantine.dev/charts/treemap/)；对应 `P11-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `aspectRatio`, `autoContrast`, `children`, `data`, `dataKey`, `height`, `strokeColor`, `strokeWidth`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `treemapProps`, `valueFormatter`, `withTooltip`

**官方功能章节：** Usage；Nested data；Node color；autoContrast；Node stroke；Disable tooltip

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `aspectRatio`, `autoContrast`, `children`, `dataKey`, `height`, `strokeColor`, `strokeWidth`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `treemapProps`, `valueFormatter`, `withTooltip`

### SunburstChart — C:SunburstChart

来源：[@mantine/charts](https://mantine.dev/charts/sunburst-chart/)；对应 `P11-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `data`, `dataKey`, `endAngle`, `gap`, `innerRadius`, `size`, `startAngle`, `strokeColor`, `sunburstChartProps`, `tooltipAnimationDuration`, `tooltipProps`, `valueFormatter`, `withLabels`, `withTooltip`

**官方功能章节：** Usage；Data format；Node color；Stroke color；Chart size；Gap；Labels；Disable tooltip

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `children`, `dataKey`, `endAngle`, `gap`, `innerRadius`, `startAngle`, `strokeColor`, `sunburstChartProps`, `tooltipAnimationDuration`, `tooltipProps`, `valueFormatter`, `withLabels`, `withTooltip`

### SankeyChart — C:SankeyChart

来源：[@mantine/charts](https://mantine.dev/charts/sankey-chart/)；对应 `P11-08`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `colors`, `data`, `height`, `iterations`, `linkColor`, `linkCurvature`, `linkOpacity`, `nodeColor`, `nodePadding`, `nodeWidth`, `sankeyProps`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `valueFormatter`, `withTooltip`

**官方功能章节：** Usage；Node color；Custom colors；Node width and padding；Link opacity；Disable tooltip

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `children`, `colors`, `height`, `iterations`, `linkColor`, `linkCurvature`, `linkOpacity`, `nodeColor`, `nodePadding`, `nodeWidth`, `sankeyProps`, `textColor`, `tooltipAnimationDuration`, `tooltipProps`, `valueFormatter`, `withTooltip`

### MatrixChart — C:MatrixChart

来源：[@mantine/charts](https://mantine.dev/charts/matrix-chart/)；对应 `P11-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `cellRadius`, `cellSize`, `colors`, `data`, `domain`, `emptyColor`, `fontSize`, `gap`, `getCellProps`, `getTooltipLabel`, `legendLabels`, `tooltipProps`, `withLegend`, `withTooltip`, `withXLabels`, `withYLabels`, `xLabels`, `xLabelsHeight`, `xLabelsPosition`, `xLabelsRotation`, `yLabels`, `yLabelsWidth`

**官方功能章节：** Usage；Data format；Labels；X-axis labels rotation；Tooltip；Colors；Values domain；Cell size and gap；Pass props to cells；Legend；Accessibility

**本地实现：** [src/plot_hierarchy.rs](../src/plot_hierarchy.rs)、[src/plot_model.rs](../src/plot_model.rs)、[src/plot_scene.rs](../src/plot_scene.rs)、[src/plot_table.rs](../src/plot_table.rs)、[src/plot_widget.rs](../src/plot_widget.rs)

**本地公开入口（同族）：** `build`, `chart_theme`, `class`, `data`, `data_table`, `default_options`, `enabled`, `focus`, `frame`, `keys`, `legend`, `name`, `new`, `node_ref`, `on`, `on_options_change`, `on_selection_change`, `options`, `plot`, `pointer_selection`, `sample_points`, `selection`, `size`, `style`, `theme`, `unstyled`, `validate`, `values`

**需要语义映射的属性候选：** `cellRadius`, `cellSize`, `colors`, `domain`, `emptyColor`, `fontSize`, `gap`, `getCellProps`, `getTooltipLabel`, `legendLabels`, `tooltipProps`, `withLegend`, `withTooltip`, `withXLabels`, `withYLabels`, `xLabels`, `xLabelsHeight`, `xLabelsPosition`, `xLabelsRotation`, `yLabels`, `yLabelsWidth`

### Schedule — T:Schedule

来源：[@mantine/schedule](https://mantine.dev/schedule/schedule/)；对应 `P12-01`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayViewProps`, `defaultDate`, `defaultView`, `events`, `labels`, `layout`, `locale`, `mobileMonthViewProps`, `mode`, `monthViewProps`, `onAllDaySlotClick`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `radius`, `recurrenceExpansionLimit`, `renderEventBody`, `view`, `weekViewProps`, `withAgenda`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withInteractiveBackgroundEvents`, `yearViewProps`

**官方功能章节：** Usage；Controlled state；View-specific props；Default view；Custom header；Agenda view；Drag and drop；Create and update events；External drag and drop；Bidirectional drag and drop；Event resize；Drop validation；Recurring events；Background events；Responsive layout；View components

**本地实现：** [src/schedule/mod.rs](../src/schedule/mod.rs)、[src/schedule_layout/mod.rs](../src/schedule_layout/mod.rs)、[src/schedule_model.rs](../src/schedule_model.rs)、[src/schedule_recurrence.rs](../src/schedule_recurrence.rs)

**本地公开入口（同族）：** `agenda_view`, `all_day`, `application`, `as_str`, `at`, `civil`, `class`, `column_width`, `columns`, `conflicts`, `contains`, `data`, `day_view`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `index_on`, `items`, `key`, `layout_day`, `layout_timeline`, `locale`, `mobile_month_view`, `month`, `month_view`, `name`, `new`, `node_ref`, `on`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `placements`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resources`, `schedule`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `week_start`, `week_view`, `window`, `year`, `year_view`, `zone`

**需要语义映射的属性候选：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayViewProps`, `defaultDate`, `defaultView`, `labels`, `layout`, `mobileMonthViewProps`, `mode`, `monthViewProps`, `onAllDaySlotClick`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `radius`, `recurrenceExpansionLimit`, `renderEventBody`, `weekViewProps`, `withAgenda`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withInteractiveBackgroundEvents`, `yearViewProps`

### DayView — T:DayView

来源：[@mantine/schedule](https://mantine.dev/schedule/day-view/)；对应 `P12-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allDaySlotHeight`, `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventOverlapMode`, `eventOverlapRaiseDelay`, `eventResizeInterval`, `events`, `getCurrentTime`, `getTimeSlotProps`, `headerFormat`, `highlightBusinessHours`, `intervalMinutes`, `labels`, `locale`, `mode`, `moreEventsProps`, `nextControlProps`, `onAllDaySlotClick`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `scrollAreaProps`, `slotHeight`, `slotLabelFormat`, `slotLabelInterval`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `withAgenda`, `withAllDaySlot`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withSubHourGridLines`

**官方功能章节：** Usage；Time range；Start scroll time；Interval minutes；Slot label interval；Sub-hour grid lines；All-day events；Overlapping events；Current time indicator；Current time indicator in a different timezone；Business hours；Business hours colors；Custom time slot props；Slot height；Without header；Custom header；Header format；Slot label format；Radius；Overlap mode；Drag and drop；Drag interval；External drag and drop；Bidirectional drag and drop；Selective drag permissions；Event resize；Resize interval；Selective resize permissions；Custom event rendering；Full event customization；Recurring events；Background events；Interactive background events；Resizable background events；Custom background event styles with drag and drop blocking；Agenda view；Static mode；Custom labels；Localization；Controlled date；View change；Create and update events；Responsive styles；Accessibility

**本地实现：** [src/schedule/timeline.rs](../src/schedule/timeline.rs)、[src/schedule_layout/mod.rs](../src/schedule_layout/mod.rs)

**本地公开入口（同族）：** `at`, `columns`, `layout_day`, `layout_timeline`, `new`, `placements`

**需要语义映射的属性候选：** `allDaySlotHeight`, `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventOverlapMode`, `eventOverlapRaiseDelay`, `eventResizeInterval`, `events`, `getCurrentTime`, `getTimeSlotProps`, `headerFormat`, `highlightBusinessHours`, `intervalMinutes`, `labels`, `locale`, `mode`, `moreEventsProps`, `nextControlProps`, `onAllDaySlotClick`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `scrollAreaProps`, `slotHeight`, `slotLabelFormat`, `slotLabelInterval`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `withAgenda`, `withAllDaySlot`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withSubHourGridLines`

### MonthView — T:MonthView

来源：[@mantine/schedule](https://mantine.dev/schedule/month-view/)；对应 `P12-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `consistentWeeks`, `date`, `events`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `maxEventsPerDay`, `mode`, `monthYearSelectProps`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onExternalEventDrop`, `onSlotDragEnd`, `onViewChange`, `onWeekNumberClick`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `scrollAreaProps`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withAgenda`, `withDragSlotSelect`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

**官方功能章节：** Usage；With week numbers；Without week days；Hide weekend days；First day of week；Weekday format；Consistent weeks；Highlight today；Without outside days；Radius；Without header；Custom header；Max events per day；Many events；Drag and drop；External drag and drop；Bidirectional drag and drop；Full event customization；Timed events；Recurring events；Background events；Interactive background events；Agenda view；Static mode；Localization；Create and update events；Responsive styles；Accessibility

**本地实现：** [src/schedule/calendar.rs](../src/schedule/calendar.rs)

**本地公开入口（同族）：** 

**需要语义映射的属性候选：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `consistentWeeks`, `date`, `events`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `maxEventsPerDay`, `mode`, `monthYearSelectProps`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onExternalEventDrop`, `onSlotDragEnd`, `onViewChange`, `onWeekNumberClick`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `scrollAreaProps`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withAgenda`, `withDragSlotSelect`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

### WeekView — T:WeekView

来源：[@mantine/schedule](https://mantine.dev/schedule/week-view/)；对应 `P12-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allDaySlotHeight`, `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventOverlapMode`, `eventOverlapRaiseDelay`, `eventResizeInterval`, `events`, `firstDayOfWeek`, `forceCurrentTimeIndicator`, `getCurrentTime`, `getTimeSlotProps`, `highlightBusinessHours`, `highlightToday`, `intervalMinutes`, `labels`, `locale`, `mode`, `nextControlProps`, `onAllDaySlotClick`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderWeekLabel`, `scrollAreaProps`, `slotHeight`, `slotLabelFormat`, `slotLabelInterval`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `weekLabelFormat`, `weekdayFormat`, `weekendDays`, `withAgenda`, `withAllDaySlots`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withSubHourGridLines`, `withWeekNumber`, `withWeekendDays`

**官方功能章节：** Usage；Controlled date；Time range；Start scroll time；Interval minutes；Slot label interval；Sub-hour grid lines；First day of week；Weekday format；Without weekend days；Highlight today；Without week number；Current time indicator；Force current time indicator；Current time indicator in a different timezone；Without all-day slots；All-day events；Without header；View change callback；Custom header；Week label format；Slot label format；Slot height；Radius；Business hours；Business hours per day；Business hours colors；Custom time slot props；Overlapping events；Overlap mode；Drag and drop；Drag interval；Restrict draggable events；External drag and drop；Bidirectional drag and drop；Event resize；Resize interval；Restrict resizable events；Full event customization；Custom event body；Recurring events；Background events；Interactive background events；Resizable background events；Custom background event styles with drag and drop blocking；Agenda view；Static mode；Localization；Create and update events；Responsive styles；Accessibility

**本地实现：** [src/schedule/timeline.rs](../src/schedule/timeline.rs)、[src/schedule_layout/mod.rs](../src/schedule_layout/mod.rs)

**本地公开入口（同族）：** `at`, `columns`, `layout_day`, `layout_timeline`, `new`, `placements`

**需要语义映射的属性候选：** `allDaySlotHeight`, `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventOverlapMode`, `eventOverlapRaiseDelay`, `eventResizeInterval`, `events`, `firstDayOfWeek`, `forceCurrentTimeIndicator`, `getCurrentTime`, `getTimeSlotProps`, `highlightBusinessHours`, `highlightToday`, `intervalMinutes`, `labels`, `locale`, `mode`, `nextControlProps`, `onAllDaySlotClick`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderWeekLabel`, `scrollAreaProps`, `slotHeight`, `slotLabelFormat`, `slotLabelInterval`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `weekLabelFormat`, `weekdayFormat`, `weekendDays`, `withAgenda`, `withAllDaySlots`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withSubHourGridLines`, `withWeekNumber`, `withWeekendDays`

### YearView — T:YearView

来源：[@mantine/schedule](https://mantine.dev/schedule/year-view/)；对应 `P12-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `date`, `events`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `mode`, `monthLabelFormat`, `monthYearSelectProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onMonthClick`, `onViewChange`, `onWeekNumberClick`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderDay`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withHeader`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

**官方功能章节：** Usage；With week numbers；Without week days；Render day；Hide weekend days；Hide outside days；First day of week；Weekday format；Highlight today；Without header；Custom header；Recurring events；Static mode；Localization；Responsive styles；Accessibility；OTHER COMPONENTS AND FEATURES；Releases cycle；Previous versions documentation；Project maintenance；Stats；Mantine logo；Supported browsers；CSS features that contribute to browser support；Polyfills；Ways to contribute；Contributing workflow；Get started with Mantine locally；npm scripts；Get started with a template；Community templates；Framework guide；Can I use Mantine with create-react-app?；Get started without framework；Set up VS Code；Learn；Support Mantine；Mantine for Figma；License；Installation；oxlint configuration；oxfmt configuration；Running oxlint and oxfmt；Rules and source code；Get help；Report an issue；Get help on GitHub Discussions；Support Mantine；Migration to native CSS；Vanilla extract integration；System color scheme support；Built-in color scheme manager；CSS modules and PostCSS preset；Global styles；Mantine as a headless UI library；createTheme function；Components extend functions；classNames based on component props；Components CSS variables；New variants system；New sizes system；theme.variantColorResolver；rem units scaling；color prop improvements；Components classes；Theme object changes；Colors generator；New setup for RTL；React 18+ only；left and right section；NumberInput changes；Loader changes；Progress changes；Table changes；Group changes；Tabs changes；Button changes；AppShell changes；SimpleGrid changes；Grid changes；Image changes；Spotlight changes；Carousel changes；@mantine/prism deprecation；Fieldset component；Combobox component；TagsInput component；withErrorStyles prop；hiddenFrom and visibleFrom props；Other changes；CSS layers；renderRoot prop；Improved ESM support；CSS variables in style prop；form.setInitialValues；Tree component；form.getInputNode；Container queries in SimpleGrid；Checkbox and Radio indicators；Checkbox and Radio cards；bd style prop；withProps function；Avatar initials；BubbleChart component；BarChart waterfall type；LineChart gradient type；Right Y axis；RadarChart legend；TagsInput acceptValueOnBlur；Transition delay；Documentation updates；Other changes；Notifications at any position；Subscribe to notifications state；SemiCircleProgress component；Tree checked state；Disable specific features in postcss-preset-mantine；Help Center updates；Other changes；Container queries support in Grid；CompositeChart component；Points labels；BarChart: Mixed stacks；BarChart: Minimum bar size；New demo: Custom Tabs variant；New demo: Custom slider；Help Center updates；Other changes；AngleSlider component；RadialBarChart component；FunnelChart component；Modal.Stack and Drawer.Stack components；useModalsStack/useDrawersStack hooks；Restrict Slider selection to marks；BarChart SVG pattern fill；Help center updates；Other changes；Support Mantine development；use-radial-move hook；BarChart color based on value；Button.GroupSection and ActionIcon.GroupSection；Table vertical variant；Table tabular numbers；Update function in modals manager；useForm submitting state；useForm onSubmitPreventDefault option；Subtle RichTextEditor variant；onExitTransitionEnd and onEnterTransitionEnd；Week numbers in DatePicker；New demo: BarChart with overlay；Variants types augmentation；Help Center updates；use-scroll-spy hook；TableOfContents component；Input.ClearButton component；Popover with overlay；Container queries in Carousel；RangeSlider restrictToMarks；Pagination withPages prop；useForm touchTrigger option；Help Center updates；Other changes；Portal reuseTargetNode prop；use-form formRootRule；isJSONString and isNotEmptyHTML form validators；Popover onDismiss；MantineProvider env；use-file-dialog hook；Remix deprecation；Help center updates；Other changes；Community templates；NumberFormatter component；Form actions；Table data prop；Table sticky header；Usage with Sass；Inline loaders；lightHidden and darkHidden props；light-root and dark-root mixins；Documentation updates；Other changes；smaller-than and larger-than mixins；Form schema resolvers packages；rem/em functions improvements；component and renderRoot props for non-polymorphic components；Outline Checkbox and Radio variant；HueSlider and AlphaSlider components；Button loading state animation；Drawer offset；z-index CSS variables；Improved dark color scheme colors；Documentation updates；Other changes；@mantine/charts；AreaChart component；LineChart component；BarChart component；Sparkline component；OKLCH colors support；autoContrast；Color functions improvements；enhanceGetInputProps；form.initialize；valibot form resolver；ScrollArea scrollbars prop；Title lineClamp prop；Primary color CSS variables；Help center；Documentation updates；Other changes；DonutChart component；PieChart component；@mantine/dates value formatter；@mantine/dates consistent weeks；Charts series label；Charts value formatter；Headings text wrap；mod prop；Documentation updates；Help center updates；Other changes；Container queries support；RadarChart component；FocusTrap.InitialFocus component；New MantineProvider props；HeadlessMantineProvider；Sparkline trendColors；RichTextEditor tasks extension；renderOption prop；Styles improvements；Pass props to inner recharts components；PieChart percent labels；Documentation updates；Help center updates；Other changes；Virtual colors；FloatingIndicator component；ScatterChart component；colorsTuple function；use-mutation-observer hook；use-state-history hook；Axis labels；Documentation updates；Help center updates；Other changes；Auto convert px to rem in .css files；Uncontrolled form mode；form.getValues；form.watch；Customize Popover middlewares；use-fetch hook；use-map hook；use-set hook；use-debounced-callback hook；use-throttled-state hook；use-throttled-value hook；use-throttled-callback hook；use-orientation hook；use-is-first-render hook；Documentation updates；Other changes；@mantine/emotion package；React 18.3 support；use-field hook；Custom PostCSS mixins；use-matches hook；BarChart value label；Documentation updates；Other changes；Support Mantine development；Migration guide；Granular global styles exports；Menu with submenus；Popover hideDetached；Date values as strings；DatesProvider timezone；TimePicker component；DateTimePicker component changes；TimeValue component；TimeGrid component；Heatmap component；CodeHighlight changes；CodeHighlight with shiki；Carousel changes；Switch withThumbIndicator；Theme types augmentation；Other changes；Support Mantine development；DatePicker presets；Calendar headerControlsOrder；Popover middlewares improvements；use-long-press hook；Reference area support in charts；use-form resetField handler；TagsInput isDuplicate prop；Slider domain prop；RangeSlider pushOnOverlap prop；Hooks types exports；zod v4 with use-form；Documentation updates；Other changes；Support Mantine development；Styles API attributes；Container grid strategy；Tooltip target；HoverCard.Group；use-selection hook；autoSelectOnBlur prop；Source edit mode in RichTextEditor；Recharts 3 support；Other changes；Support Mantine development；MiniCalendar component；Progress vertical orientation；Heatmap splitMonths；Improved clearable prop handling；Tiptap 3 support；LLMs.txt；Other changes；Support Mantine development；Migration guide；Peer dependencies requirements updates；New @mantine/schedule package；Collapse horizontal orientation；use-collapse and use-horizontal-collapse hooks；use-floating-window hook；FloatingWindow component；OverflowList component；Marquee component；Scroller component；use-scroller hook；BarsList component；Card horizontal orientation；Checkbox.Group and Switch.Group maxSelectedValues；Inputs loading state；renderPill in MultiSelect and TagsInput；Clear section mode；use-form async validation；use-form TransformedValues type argument；Generic components；Combobox virtualization；Highlight per-term colors；Highlight whole-word matching；Pagination and use-pagination startValue；Grid improvements；Slider vertical orientation；SimpleGrid improvements；Calendar fullWidth prop；Namespace types exports；Font weights；@mantine/mcp-server package；Mantine skills for Claude Code and Codex；New use-form validators；Standard Schema support in use-form；AppShell static mode；ScrollArea startScrollPosition；useEffectEvent migration in hooks；React 19 Activity for keepMounted；Documentation updates；Other changes；Support Mantine development；deduplicateInlineStyles；use-mask hook；MaskInput component；Treemap component；TimePicker duration type；Heatmap legend；MonthPicker and YearPicker presets；use-roving-index hook；Tree drag and drop；Tree async loading；Tree search and filtering；Tree connecting lines；Tree virtualization；Tree checkStrictly mode；Slider startPointValue；WeekView forceCurrentTimeIndicator；New demo: MonthView events rendering；Other changes；Support Mantine development；TreeSelect component；Tree select Combobox examples；Notifications swipe dismissal；use-drag hook；InlineDateTimePicker component；DateTimePicker range support；DateTimePicker valueFormat function；RollingNumber component；MaskInput improvements；SankeyChart component；Reorder pills in MultiSelect and TagsInput；Restrict Tree drop targets；Tree drag handle；Shared default props for all inputs；Per-day business hours in WeekView；Support Mantine development；Pagination responsive layout；Text textWrap prop；use-splitter hook；Splitter component；CodeHighlight line numbers；OverflowList collapseFrom；Textarea bottomSection；Combobox floatingHeight；Menu submenu safe polygon；Menu search；Menu checkbox and radio items；Menu context menu；Popover context menu；Menu type-ahead navigation；Highlight accent and case insensitive matching；PieChart and DonutChart labelsType="name"；Tooltip merge arrow position；Popover preventPositionChangeWhenVisible default；Schedule getCurrentTime；Support Mantine development；ComboboxPopover component；DataList component；EmptyState component；Menubar component；ResourcesDayView component；ResourcesWeekView component；ResourcesMonthView component；ResourcesSchedule component；AgendaView component；withAgenda prop for DayView, WeekView, MonthView and Schedule；MonthView hide weekend days；DayView and WeekView sub-hour grid lines；Input success state；Splitter CSS units；Notifications priority；Legend support for PieChart, DonutChart and FunnelChart；autoContrast support for virtual colors；Other changes；Support Mantine development；Migration to oxc；Native level select in date pickers；Timeline opposite and alternate content；FloatingWindow resize handle；Cascader component；SunburstChart component；BulletChart component；Charts keyboard navigation；Charts brush；Heatmap month labels position；Accordion disable collapse；ScrollArea vertical scrollbar position；Schedule intervals larger than one hour；New example: Table virtualization；Interactive tooltip；Support Mantine development；@mantine/lightbox package；Notifications custom rendering；Notifications stacked layout；ActionBar component；RichTextEditor table controls；RichTextEditor Details control；RichTextEditor InvisibleCharacters control；GaugeChart component；WaffleChart component；MatrixChart component；CandlestickChart component；Charts reference areas；Charts reference dots；AreaChart streamgraph；ScatterChart right Y axis；Stepper labelPosition；Cascader safe area polygon；YearView renderDay；ResourcesMonthView event resize；Schedule drag and resize intervals；Dropzone react-dropzone 20；Other changes；Support Mantine development；@mantine/mcp-server documentation coverage；Tour component；Toolbar component；Toggle component；JsonViewer component；Light Checkbox and Radio variant；RichTextEditor image upload；Schedule event overlap mode；Schedule drop validation；TreeSelect renderPill；Tooltip hideDetached；Notifications auto close progress；Notifications promise；AppShell resizable sections；Templates and guides；Other changes；Releases；Releases；FREQUENTLY ASKED QUESTIONS；Attribute selector；Are Mantine components accessible?；How are Mantine components tested for accessibility?；Is there anything I need to do on my side to make my app accessible?；Data structure；Value/label data split；Difference between Autocomplete and Select；Change body background with CSS；Change body background with CSS variable；If you need Astro, you do not need a component library；I need Mantine, what should I use instead?；No! I need Astro and Mantine!；Deprecation notice；Usage without postcss-preset-mantine；Ejecting Create React App；Ejected CRA example；Carousel component looks broken；That's it! It works now!；How Mantine applies color scheme；Flash of inaccurate color scheme；Example hydration warning；Why do I get hydration warning?；How to fix hydration warning?；What is mantineHtmlProps?；Does hydration warning indicate a problem?；How suppressHydrationWarning works?；Getting started；Selecting one option (Select, Autocomplete)；Selecting multiple options (MultiSelect, TagsInput)；Searching；Dropdown opened state；How @mantine/form works；use-uncontrolled hook；Full example；Community packages；Is it planned to add DataGrid component to Mantine?；Disable all inputs with fieldset；Disable all inputs with enhanceGetInputProps；data-mantine-color-scheme attribute；Adding support for disabled JavaScript；Example；data- attributes；Inline styles；CSS variables；Example of the issue；Source of the issue；How to fix the issue；Get input DOM node with use-form；Focus first input with error on form submit；Ways to contribute；How to contribute to the codebase；How to get started with Mantine locally；mantine.dev website；How can I build the same footer?；&:hover；@mixin hover；Is there a way to add hover styles inline in JSX?；Get color scheme value in component；Get color scheme value outside of component；I want to hide/show some elements based on color scheme value；Installation；Configuration；Mantine Stylelint config；Additional tools；Update dependencies with npm-check-updates；ncu script；Update dependencies manually；How do I know which version is the latest?；Change focus styles of a single input；Change focus styles of all inputs；How Mantine color scheme works；Can I get color scheme value in JavaScript?；lightHidden and darkHidden props；Changing component styles based on color scheme；What is color scheme?；I want to add a custom color scheme. Can I do that?；How can I customize colors with theme?；What is use-form list?；Form lists limitation；How use-local-storage hook works；Reading value in first render；Why is the MultiSelect placeholder not removed when values are selected?；How to remove the MultiSelect placeholder when values are selected?；Native browser validation；Is there a workaround to use native browser validation?；What should I do instead?；What are nested inline styles?；Styles in Mantine components；Why are nested inline styles not supported?；What is the alternative?；I still want to use nested inline styles；Nested popovers；How to fix；Loading fonts with next/font package；Loading fonts without next/font package；Load fonts from Google Fonts；Notifications component；How to fix；Do you have a comparison with other libraries?；How should I decide which library to use?；What is polymorphic component?；Polymorphic components types；Why I cannot use one polymorphic component in component prop of another polymorphic component?；Getting started；Testing example；Failing tests；Fixing failing tests；How to test that the modal is opened/closed?；What are PostCSS functions?；Can I use PostCSS functions in inline styles?；Virtual colors；What are private CSS variables?；Should I use private CSS variables to apply styles?；But what should I do instead?；Loading local fonts；Load fonts from Google Fonts；Roadmap. Where is it?；How can I know what is coming next?；When will the next version be released?；Will Mantine be maintained in the future?；use-form submit handlers；Scroll to the top of the form on submit；Can I use SegmentedControl with an initial empty value?；Can I deselect a value in SegmentedControl?；What should I use instead if I need deselect functionality?；I want a component that looks like SegmentedControl but can be deselected；Searchable select；Autocomplete；Options filtering；Example with a custom filter function that matches options by words instead of a letter sequence:；Example with fuse.js；Difference between server and client components；Can I use Mantine components as server components?；Should I add 'use client' directive to all files?；Does 'use client' directive affect SEO?；Error: hook is not a function；Error: function cannot be passed as children；Error: hook usage in server component；Error: compound components in server component；Error: Attempted to call extend() from the server；How Mantine styles work；Styles overriding conflicts；CSS layers；What is a template；How to submit a template；Prerequisites；Getting started with a template；Without GitHub account；How Mantine colors are defined and used；Why is it required to have 10 shades per color?；Can I have just one shade per color?；Can I have more than 10 shades per color?；I prefer a third-party styling solution, can I use Mantine with it?；Is there any specific setup for Tailwind CSS?；Why are my buttons transparent?；How to fix it?；I do not use Tailwind, what else can cause this issue?；Loading local fonts；Load fonts from Google Fonts；Can I use Mantine with Vue/Svelte/Angular?；What about Preact?；Why is the Text component not automatically imported?；How can I import the Text component with VSCode?

**本地实现：** [src/schedule/calendar.rs](../src/schedule/calendar.rs)

**本地公开入口（同族）：** 

**需要语义映射的属性候选：** `date`, `events`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `mode`, `monthLabelFormat`, `monthYearSelectProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onMonthClick`, `onViewChange`, `onWeekNumberClick`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderDay`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withHeader`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

### MobileMonthView — T:MobileMonthView

来源：[@mantine/schedule](https://mantine.dev/schedule/mobile-month-view/)；对应 `P12-03`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `consistentWeeks`, `date`, `defaultSelectedDate`, `events`, `eventsHeaderFormat`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `mode`, `onDateChange`, `onDayClick`, `onEventClick`, `onSelectedDateChange`, `onWeekNumberClick`, `onYearClick`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderHeader`, `selectedDate`, `weekdayFormat`, `weekendDays`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

**官方功能章节：** Usage；With week numbers；With outside days；Hide weekend days；Highlight today；First day of week；Custom header；Consistent weeks；Weekday format；Without week days；Static mode；Localization；Accessibility

**本地实现：** [src/schedule/calendar.rs](../src/schedule/calendar.rs)

**本地公开入口（同族）：** 

**需要语义映射的属性候选：** `consistentWeeks`, `date`, `defaultSelectedDate`, `events`, `eventsHeaderFormat`, `firstDayOfWeek`, `getDayProps`, `getWeekNumberProps`, `highlightToday`, `labels`, `locale`, `mode`, `onDateChange`, `onDayClick`, `onEventClick`, `onSelectedDateChange`, `onWeekNumberClick`, `onYearClick`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderHeader`, `selectedDate`, `weekdayFormat`, `weekendDays`, `withOutsideDays`, `withWeekDays`, `withWeekNumbers`, `withWeekendDays`

### AgendaView — T:AgendaView

来源：[@mantine/schedule](https://mantine.dev/schedule/agenda-view/)；对应 `P12-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Usage；All-day and multi-day events；Date formats；Custom event rendering；Event click；Localization；Static mode

**本地实现：** [src/schedule/mod.rs](../src/schedule/mod.rs)

**本地公开入口（同族）：** `agenda_view`, `class`, `column_width`, `day_view`, `default_selection`, `direction`, `enabled`, `focus`, `grid_ref`, `locale`, `mobile_month_view`, `month_view`, `name`, `node_ref`, `on`, `on_selection_change`, `on_window_request`, `readonly`, `schedule`, `selection`, `size`, `style`, `theme`, `time_window`, `unstyled`, `view`, `viewport_height`, `viewport_ref`, `week_start`, `week_view`, `year_view`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### ResourcesDayView — T:ResourcesDayView

来源：[@mantine/schedule](https://mantine.dev/schedule/resources-day-view/)；对应 `P12-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventResizeInterval`, `events`, `getCurrentTime`, `groupLabelWidth`, `groups`, `headerFormat`, `highlightBusinessHours`, `intervalMinutes`, `labels`, `locale`, `maxEventsPerTimeSlot`, `minEventSize`, `mode`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `resources`, `rowHeight`, `scrollAreaProps`, `slotLabelFormat`, `slotWidth`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`

**官方功能章节：** Usage；All-day events；Interactive background events；Resizable background events；Time range and intervals；Multi-hour intervals；Start scroll time；Slot width and row height；Custom resource label；Resource groups；Custom event rendering；Drag and drop；Drag interval；External drag and drop；Bidirectional drag and drop；Event resize；Resize interval；Event form；Business hours；Current time indicator；Current time indicator in a different timezone；Event permissions；Recurring events；Max events per time slot；Radius；Scroll area props；Localization；Static mode

**本地实现：** [src/schedule/resources/drag.rs](../src/schedule/resources/drag.rs)、[src/schedule/resources/mod.rs](../src/schedule/resources/mod.rs)、[src/schedule/resources/timeline.rs](../src/schedule/resources/timeline.rs)、[src/schedule_layout/resource_month.rs](../src/schedule_layout/resource_month.rs)、[src/schedule_model.rs](../src/schedule_model.rs)

**本地公开入口（同族）：** `all_day`, `application`, `as_str`, `axis`, `civil`, `class`, `column_width`, `conflicts`, `contains`, `data`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `items`, `key`, `layout_month`, `locale`, `month`, `name`, `new`, `node_ref`, `on`, `on_edit`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resource_page_size`, `resources`, `resources_day_view`, `resources_month_view`, `resources_schedule`, `resources_week_view`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `window`, `year`, `zone`

**需要语义映射的属性候选：** `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventResizeInterval`, `getCurrentTime`, `groupLabelWidth`, `groups`, `headerFormat`, `highlightBusinessHours`, `intervalMinutes`, `labels`, `maxEventsPerTimeSlot`, `minEventSize`, `mode`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `rowHeight`, `scrollAreaProps`, `slotLabelFormat`, `slotWidth`, `startScrollTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`

### ResourcesWeekView — T:ResourcesWeekView

来源：[@mantine/schedule](https://mantine.dev/schedule/resources-week-view/)；对应 `P12-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventResizeInterval`, `events`, `firstDayOfWeek`, `getCurrentTime`, `groupLabelWidth`, `groups`, `highlightBusinessHours`, `highlightToday`, `intervalMinutes`, `labels`, `locale`, `maxEventsPerTimeSlot`, `minEventSize`, `mode`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `renderWeekLabel`, `resources`, `rowHeight`, `scrollAreaProps`, `slotLabelFormat`, `slotWidth`, `startScrollDateTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `weekLabelFormat`, `weekdayFormat`, `weekendDays`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withWeekendDays`

**官方功能章节：** Usage；All-day events；Interactive background events；Resizable background events；Drag and drop；Drag interval；Event form；Time range and intervals；Multi-hour intervals；Current time indicator；Current time indicator in a different timezone；First day of week；Custom week label；Business hours；Without weekend days；Custom resource label；Resource groups；Custom event rendering；Recurring events；Max events per time slot；Localization；Radius；Start scroll date time；Scroll area props；Event resize；Resize interval；Static mode

**本地实现：** [src/schedule/resources/drag.rs](../src/schedule/resources/drag.rs)、[src/schedule/resources/mod.rs](../src/schedule/resources/mod.rs)、[src/schedule/resources/timeline.rs](../src/schedule/resources/timeline.rs)、[src/schedule_layout/resource_month.rs](../src/schedule_layout/resource_month.rs)、[src/schedule_model.rs](../src/schedule_model.rs)

**本地公开入口（同族）：** `all_day`, `application`, `as_str`, `axis`, `civil`, `class`, `column_width`, `conflicts`, `contains`, `data`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `items`, `key`, `layout_month`, `locale`, `month`, `name`, `new`, `node_ref`, `on`, `on_edit`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resource_page_size`, `resources`, `resources_day_view`, `resources_month_view`, `resources_schedule`, `resources_week_view`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `window`, `year`, `zone`

**需要语义映射的属性候选：** `businessHours`, `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `endTime`, `eventDragInterval`, `eventResizeInterval`, `firstDayOfWeek`, `getCurrentTime`, `groupLabelWidth`, `groups`, `highlightBusinessHours`, `highlightToday`, `intervalMinutes`, `labels`, `maxEventsPerTimeSlot`, `minEventSize`, `mode`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `renderWeekLabel`, `rowHeight`, `scrollAreaProps`, `slotLabelFormat`, `slotWidth`, `startScrollDateTime`, `startTime`, `todayControlProps`, `viewSelectProps`, `weekLabelFormat`, `weekdayFormat`, `weekendDays`, `withCurrentTimeBubble`, `withCurrentTimeIndicator`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withInteractiveBackgroundEvents`, `withWeekendDays`

### ResourcesMonthView — T:ResourcesMonthView

来源：[@mantine/schedule](https://mantine.dev/schedule/resources-month-view/)；对应 `P12-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayWidth`, `events`, `groupLabelWidth`, `groups`, `highlightToday`, `labels`, `locale`, `maxEventsPerTimeSlot`, `mode`, `monthYearSelectProps`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `resources`, `rowHeight`, `scrollAreaProps`, `startScrollDate`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withWeekendDays`

**官方功能章节：** Usage；Static mode；Drag and drop；Event resize；Event form；Custom event rendering；Custom resource label；Resource groups；Localization；Recurring events；Max events per cell；Day width and row height；Start scroll date；Without weekend days；More events indicator；More events props；Custom header；Radius；Scroll area props

**本地实现：** [src/schedule/resources/drag.rs](../src/schedule/resources/drag.rs)、[src/schedule/resources/mod.rs](../src/schedule/resources/mod.rs)、[src/schedule/resources/timeline.rs](../src/schedule/resources/timeline.rs)、[src/schedule_layout/resource_month.rs](../src/schedule_layout/resource_month.rs)、[src/schedule_model.rs](../src/schedule_model.rs)

**本地公开入口（同族）：** `all_day`, `application`, `as_str`, `axis`, `civil`, `class`, `column_width`, `conflicts`, `contains`, `data`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `items`, `key`, `layout_month`, `locale`, `month`, `name`, `new`, `node_ref`, `on`, `on_edit`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resource_page_size`, `resources`, `resources_day_view`, `resources_month_view`, `resources_schedule`, `resources_week_view`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `window`, `year`, `zone`

**需要语义映射的属性候选：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayWidth`, `groupLabelWidth`, `groups`, `highlightToday`, `labels`, `maxEventsPerTimeSlot`, `mode`, `monthYearSelectProps`, `moreEventsProps`, `nextControlProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onViewChange`, `preventEventOverlap`, `previousControlProps`, `radius`, `recurrenceExpansionLimit`, `renderEvent`, `renderEventBody`, `renderGroupLabel`, `renderResourceLabel`, `rowHeight`, `scrollAreaProps`, `startScrollDate`, `todayControlProps`, `viewSelectProps`, `weekdayFormat`, `weekendDays`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withHeader`, `withWeekendDays`

### ResourcesSchedule — T:ResourcesSchedule

来源：[@mantine/schedule](https://mantine.dev/schedule/resources-schedule/)；对应 `P12-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayViewProps`, `defaultDate`, `defaultView`, `events`, `labels`, `locale`, `mode`, `monthViewProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `radius`, `recurrenceExpansionLimit`, `renderEventBody`, `renderResourceLabel`, `resources`, `view`, `weekViewProps`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withInteractiveBackgroundEvents`

**官方功能章节：** Usage；Controlled state；Drag and drop；Week view；View-specific props；Event form；External drag and drop；Event resize；Drop validation；Static mode

**本地实现：** [src/schedule/resources/drag.rs](../src/schedule/resources/drag.rs)、[src/schedule/resources/mod.rs](../src/schedule/resources/mod.rs)、[src/schedule/resources/timeline.rs](../src/schedule/resources/timeline.rs)、[src/schedule_layout/resource_month.rs](../src/schedule_layout/resource_month.rs)、[src/schedule_model.rs](../src/schedule_model.rs)

**本地公开入口（同族）：** `all_day`, `application`, `as_str`, `axis`, `civil`, `class`, `column_width`, `conflicts`, `contains`, `data`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `items`, `key`, `layout_month`, `locale`, `month`, `name`, `new`, `node_ref`, `on`, `on_edit`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resource_page_size`, `resources`, `resources_day_view`, `resources_month_view`, `resources_schedule`, `resources_week_view`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `window`, `year`, `zone`

**需要语义映射的属性候选：** `canDragEvent`, `canDropEvent`, `canDropExternalEvent`, `canResizeEvent`, `canResizeEventTo`, `date`, `dayViewProps`, `defaultDate`, `defaultView`, `labels`, `mode`, `monthViewProps`, `onDateChange`, `onDayClick`, `onEventClick`, `onEventDragEnd`, `onEventDragStart`, `onEventDrop`, `onEventPlacementRejected`, `onEventResize`, `onExternalEventDrop`, `onSlotDragEnd`, `onTimeSlotClick`, `onViewChange`, `preventEventOverlap`, `radius`, `recurrenceExpansionLimit`, `renderEventBody`, `renderResourceLabel`, `weekViewProps`, `withDragSlotSelect`, `withEventResize`, `withEventsDragAndDrop`, `withInteractiveBackgroundEvents`

### Recurring events — T:Recurring events

来源：[@mantine/schedule](https://mantine.dev/schedule/recurring-events/)；对应 `P12-01`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Recurring events；Series event；Override event；How rendering works；expandRecurringEvents utility；recurrenceExpansionLimit prop；Schedule demo；DayView demo；WeekView demo；MonthView demo；YearView demo

**本地实现：** [src/schedule/mod.rs](../src/schedule/mod.rs)、[src/schedule_layout/mod.rs](../src/schedule_layout/mod.rs)、[src/schedule_model.rs](../src/schedule_model.rs)、[src/schedule_recurrence.rs](../src/schedule_recurrence.rs)

**本地公开入口（同族）：** `agenda_view`, `all_day`, `application`, `as_str`, `at`, `civil`, `class`, `column_width`, `columns`, `conflicts`, `contains`, `data`, `day_view`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `index_on`, `items`, `key`, `layout_day`, `layout_timeline`, `locale`, `mobile_month_view`, `month`, `month_view`, `name`, `new`, `node_ref`, `on`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `placements`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resources`, `schedule`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `week_start`, `week_view`, `window`, `year`, `year_view`, `zone`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Events data — T:Events data

来源：[@mantine/schedule](https://mantine.dev/schedule/events-data/)；对应 `P12-01`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Overview；ScheduleEventBase；ScheduleSingleEventData；ScheduleRecurringSeriesEventData；ScheduleRecurringOverrideEventData；EventPayload；Recurring instance metadata；Callback payloads；Date and time value formats

**本地实现：** [src/schedule/mod.rs](../src/schedule/mod.rs)、[src/schedule_layout/mod.rs](../src/schedule_layout/mod.rs)、[src/schedule_model.rs](../src/schedule_model.rs)、[src/schedule_recurrence.rs](../src/schedule_recurrence.rs)

**本地公开入口（同族）：** `agenda_view`, `all_day`, `application`, `as_str`, `at`, `civil`, `class`, `column_width`, `columns`, `conflicts`, `contains`, `data`, `day_view`, `days`, `default_selection`, `direction`, `duration_nanoseconds`, `enabled`, `event`, `events`, `except_dates`, `exceptions`, `expand`, `fixed`, `focus`, `get`, `grid_ref`, `index_on`, `items`, `key`, `layout_day`, `layout_timeline`, `locale`, `mobile_month_view`, `month`, `month_view`, `name`, `new`, `node_ref`, `on`, `on_selection_change`, `on_window_request`, `overlaps`, `permissions`, `placements`, `readonly`, `recurring`, `resize_end_by_minutes`, `resize_start_by_minutes`, `resolve`, `resource`, `resource_key`, `resources`, `schedule`, `selection`, `shifted`, `shifted_minutes`, `size`, `style`, `theme`, `time_window`, `title`, `unstyled`, `validate_edit`, `validate_resources`, `view`, `viewport_height`, `viewport_ref`, `visible_resources`, `week_start`, `week_view`, `window`, `year`, `year_view`, `zone`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### CodeHighlight — H:CodeHighlight

来源：[@mantine/x](https://mantine.dev/code-highlight/code-highlight/)；对应 `P10-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** 

**本地实现：** [src/code_block.rs](../src/code_block.rs)、[src/code_model.rs](../src/code_model.rs)

**本地公开入口（同族）：** `class`, `code_block`, `code_highlight`, `copy_button`, `highlighted_lines`, `language`, `line_numbers`, `line_starts`, `name`, `node_ref`, `on`, `size`, `source`, `start_line`, `style`, `theme`, `tokenize_code`, `tokens`, `unstyled`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### JsonViewer — H:JsonViewer

来源：[@mantine/code-highlight](https://mantine.dev/code-highlight/json-viewer/)；对应 `P10-07`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allExpanded`, `collapseAllLabel`, `collapseStringsAfterLength`, `copiedLabel`, `copyLabel`, `defaultExpandDepth`, `expandAllLabel`, `expandedPaths`, `fontSize`, `groupArraysAfterLength`, `highlightItems`, `indentWidth`, `maxDisplayLength`, `onExpandedPathsChange`, `onValueSelect`, `radius`, `rootName`, `sortKeys`, `value`, `withBorder`, `withChevrons`, `withControls`, `withCopy`, `withCopyButton`, `withKeyQuotes`, `withLineNumbers`, `withQuotes`, `withSize`, `withTypes`

**官方功能章节：** Usage；Default expand depth；Long arrays and objects；Root name；Expand all / collapse all controls；Type badges and size；Copy button；Collapse long strings；Sort keys；Highlight items；All expanded；Line numbers；Chevrons；Keyboard navigation；Accessibility；GUIDES COMPONENTS AND FEATURES；Migration to @mantine/emotion；Migration to CSS modules；Global styles imports；Portal reuseTargetNode；Switch withThumbIndicator；Date string values；DatesProvider timezone；DateTimePicker timeInputProps；CodeHighlight usage；Menu data-hovered attribute；Popover hideDetached；Carousel changes；Prerequisites；Update dependencies；use-form TransformValues type；Text color prop；Light variant color changes；Form resolvers；TypographyStylesProvider；Popover and Tooltip positionDependencies prop；use-fullscreen hook changes；use-mouse hook changes；use-mutation-observer hook changes；Rename hooks types；Collapse in -> expanded；Spoiler initialState -> defaultExpanded；Grid gutter -> gap；Grid overflow="hidden" no longer required；useLocalStorage and useSessionStorage return type；useHeadroom returns object instead of boolean；Default border-radius change；Notifications pauseResetOnHover default change；Controlled components；Uncontrolled components；Key differences；When to use which；FormData and uncontrolled components；Uncontrolled use-form；Factory type；factory function；Box component；ElementProps type；useProps hook；useStyles hook；getStyles function；varsResolver；Compound components；Namespace exports；polymorphicFactory；genericFactory；clamp；lowerFirst；upperFirst；randomId；range；shallowEqual；keys；deepMerge；filterProps；isElement；isNumberLike；camelToKebabCase；getDefaultZIndex；closeOnEscape；noop；findClosestNumber；toDateString；toDateTimeString；assignTime；clampDate；Phosphor icons；Icon size；Custom icons；Is it possible to use Mantine with JavaScript?；Transforming demos code to JavaScript；Should Mantine be used with JavaScript?；Custom render；Mock Web APIs；Framework specific setup；Testing examples；Documentation；MCP server；Skills；Example prompts；Contributing；Generate new application；Installation；PostCSS setup；Setup with pages router；Setup with app router；app + pages router together；Next.js Link with polymorphic components；Server components；Compound components in server components；app router tree shaking；Troubleshooting；What is a polymorphic component；renderRoot prop；Polymorphic components as other React components；Polymorphic components as Next.js Link；Polymorphic components with generic components；Polymorphic components with react-router NavLink；Wrapping polymorphic components；Dynamic component prop；Create your own polymorphic components；Make Mantine component polymorphic；Generate new application；Installation；PostCSS setup；Setup；Add Storybook to your application；Configure addons；Theme object；Storybook preview；Generate new application；Installation；PostCSS setup；Setup；shouldRerenderOnTransaction；immediatelyRender；StarterKit changes；Import paths；Components props types；Polymorphic components props types；Namespace types；ElementProps type；MantineTheme type；MantineThemeOverride type；MantineColorScheme type；MantineSize type；Theme object declarations；Custom variants types；Generate new application；Installation；PostCSS setup；Setup；Installation；Configuration；Custom render；Example of a full setup；THEMING COMPONENTS AND FEATURES；data-mantine-color-scheme attribute；use-mantine-color-scheme hook；use-computed-color-scheme hook；Transitions during color scheme change；Color scheme value caveats；ColorSchemeScript；Auto color scheme；Color scheme manager；Default color scheme；Force color scheme；lightHidden and darkHidden props；With disabled JavaScript；Adding extra colors；Virtual colors；colorsTuple；Supported color formats；primaryColor；primaryShade；Color prop；Colors index reference；Difference between color and c props；Colors variant resolver；Colors generation；Default colors；Add custom colors types；Default props with MantineThemeProvider；Default props for compound components；useProps hook；withProps function；Usage；MantineProvider props；Usage；Theme properties；Store theme override object in a variable；Merge multiple theme overrides；use-mantine-theme hook；Default theme；Access theme outside of components；Change fonts；System fonts；Font sizes；Line heights；h1-h6 styles；STYLES COMPONENTS AND FEATURES；darken and lighten；alpha；parseThemeColor；getThemeColor；getGradient；isLightColor；luminance；Components dependencies；Global styles；Import order；Files list；Usage；How CSS modules work；Referencing global class names；Adding styles to Mantine components；Styling Mantine components without CSS modules；CSS variables not depending on color scheme；Light color scheme only variables；Dark color scheme only variables；Typography variables；Colors variables；Spacing variables；Border radius variables；Shadow variables；z-index variables；CSS variables resolver；data attributes values；Components data attributes documentation；mod prop；Caveats and support；Usage with Vite；Usage with Next.js pages router；Usage with Next.js app router；sx prop；styles prop；styles in theme；createStyles；Utilities；CSS reset；Body and :root elements styles；Static classes；Add global styles in your application；Mantine components styles；Import styles per component；Styles import order；CSS layers；How CSS layers work；Loading styles from CDN；Installation；Usage；rem/em functions；Auto convert px to rem；dark and light mixins；smaller-than and larger-than mixins；light-dark function；alpha function；lighten and darken functions；hover mixin；rtl/ltr mixins；not-rtl/not-ltr mixins；where-\* mixins；Custom mixins；Disable specific features；rem units；rem units scaling；em units；px conversions；rem and em function；Convert rem to px；rem/em functions in css files；Convert px to rem automatically in css files；Media queries；Configure breakpoints；Breakpoints variables in CSS modules；hiddenFrom and visibleFrom props；Hidden and visible from as classes；Component size based on media query；use-media-query hook；use-matches hook；Container queries；Responsive style props；DirectionProvider；dir attribute；useDirection hook；rtl mixin；Sass modules；Usage with Vite；Usage with Next.js；Supported props；Theme values；Responsive styles；Style object；Define CSS variables in style prop；Style function；Styles array；What is Styles API；Styles API selectors；classNames prop；classNames in theme.components；Components CSS variables；styles prop；Styles API based on component props；Static classes；Components classes；Attributes；Component specific props；Style props；Style prop；CSS modules；Theme tokens；CSS modules；Inline styles；Style props；Responsive style props；Components responsive props；StyleX vs CSS Modules；Installation；Vitest；Styling components；Styles API；Components CSS variables；Theme tokens；Light and dark color schemes；Responsive styles；Dynamic styles；CSS layers；Using Mantine as a headless UI library；HeadlessMantineProvider；unstyled prop；Vanilla extract vs CSS Modules；Installation；Templates；Theming；Styling；rem and em；light and dark selectors；largerThan and smallerThan；rtl selector；Adding custom variants；Custom variants types；variantColorResolver；Sizes with components CSS variables；Sizes with data-size attribute；Sizes with static CSS variables；X COMPONENTS AND FEATURES

**本地实现：** [src/json_tree_model.rs](../src/json_tree_model.rs)、[src/json_viewer/mod.rs](../src/json_viewer/mod.rs)、[src/json_viewer/tree_mode.rs](../src/json_viewer/tree_mode.rs)

**本地公开入口（同族）：** `class`, `copy_button`, `default_expanded`, `enabled`, `expanded`, `focus`, `json_viewer`, `limits`, `name`, `node_ref`, `nodes`, `on`, `on_expanded_change`, `parse_json_tree`, `search`, `size`, `source`, `style`, `theme`, `tree_navigation`, `unstyled`, `validate`, `value`

**需要语义映射的属性候选：** `allExpanded`, `collapseAllLabel`, `collapseStringsAfterLength`, `copiedLabel`, `copyLabel`, `defaultExpandDepth`, `expandAllLabel`, `expandedPaths`, `fontSize`, `groupArraysAfterLength`, `highlightItems`, `indentWidth`, `maxDisplayLength`, `onExpandedPathsChange`, `onValueSelect`, `radius`, `rootName`, `sortKeys`, `withBorder`, `withChevrons`, `withControls`, `withCopy`, `withCopyButton`, `withKeyQuotes`, `withLineNumbers`, `withQuotes`, `withSize`, `withTypes`

### Notifications system — X:Notifications system

来源：[@mantine/notifications](https://mantine.dev/x/notifications/)；对应 `P5-06`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `allowDragDismiss`, `allowScrollDismiss`, `autoClose`, `containerWidth`, `layout`, `limit`, `notificationMaxHeight`, `pauseResetOnHover`, `portalProps`, `position`, `renderNotification`, `store`, `transitionDuration`, `withAutoCloseProgress`, `withinPortal`, `zIndex`

**官方功能章节：** Installation；Do not forget to import styles；Functions；Notification props；Dismiss interactions；Customize notification styles；Notifications container position；Limit and queue；Priority；Remove notifications from state and queue；Update notification；Promise；Auto close；Auto close progress；Pause auto close on hover；Custom notification rendering；Stacked layout；Subscribe to notifications state

**本地实现：** [src/notifications.rs](../src/notifications.rs)

**本地公开入口（同族）：** `action`, `announcement`, `capacity`, `class`, `clear`, `clock`, `closable`, `close_label`, `content`, `dismiss`, `name`, `new`, `node_ref`, `notification`, `notification_request`, `notifications`, `on_close`, `on_ready`, `pause`, `pause_on_hover`, `pending`, `severity`, `show`, `style`, `theme`, `timeout`, `title`, `toaster`, `unstyled`, `update`, `visible`, `visible_limit`

**需要语义映射的属性候选：** `allowDragDismiss`, `allowScrollDismiss`, `autoClose`, `containerWidth`, `layout`, `limit`, `notificationMaxHeight`, `pauseResetOnHover`, `portalProps`, `position`, `renderNotification`, `store`, `transitionDuration`, `withAutoCloseProgress`, `withinPortal`, `zIndex`

### Spotlight — X:Spotlight

来源：[@mantine/spotlight](https://mantine.dev/x/spotlight/)；对应 `P4-10`；原生有限能力状态 `model-only`。

**官方 Props / 子部件 Props：** `actions`, `centered`, `children`, `clearQueryOnClose`, `closeOnActionTrigger`, `closeOnClickOutside`, `closeOnEscape`, `closeSpotlightOnTrigger`, `description`, `dimmedSections`, `disabled`, `error`, `filter`, `forceOpened`, `fullScreen`, `highlightColor`, `highlightQuery`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `keywords`, `label`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `lockScroll`, `maxHeight`, `multiline`, `nothingFound`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onQueryChange`, `onSpotlightClose`, `onSpotlightOpen`, `overlayProps`, `padding`, `pointer`, `portalProps`, `query`, `radius`, `removeScrollProps`, `required`, `returnFocus`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `scrollAreaComponent`, `scrollAreaProps`, `scrollable`, `searchProps`, `shadow`, `shortcut`, `size`, `stackId`, `store`, `success`, `tagsToIgnore`, `title`, `transitionProps`, `trapFocus`, `triggerOnContentEditable`, `withAria`, `withErrorStyles`, `withOverlay`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `xOffset`, `yOffset`, `zIndex`

**官方功能章节：** Installation；Usage；Actions；Spotlight store；Keyboard shortcuts；Limit prop；Custom filter function；Fuzzy search with fuse.js；Scrollable actions list；Actions groups；Compound components；Fixed elements offset

**本地实现：** [../glory/crates/render/src/keyboard.rs](../../glory/crates/render/src/keyboard.rs)、[src/command/mod.rs](../src/command/mod.rs)

**本地公开入口（同族）：** `action`, `actions`, `cancel`, `class`, `close`, `command`, `command_action`, `command_group`, `default_open`, `default_search`, `description`, `disabled`, `enabled`, `focus`, `group`, `keywords`, `label`, `load_with`, `name`, `new`, `node_ref`, `on`, `on_action`, `on_open_change`, `on_ready`, `on_search_change`, `open`, `refresh`, `register_shortcut`, `results`, `search`, `size`, `spotlight`, `style`, `theme`, `toggle`, `typeahead`, `unstyled`

**需要语义映射的属性候选：** `centered`, `children`, `clearQueryOnClose`, `closeOnActionTrigger`, `closeOnClickOutside`, `closeOnEscape`, `closeSpotlightOnTrigger`, `dimmedSections`, `error`, `filter`, `forceOpened`, `fullScreen`, `highlightColor`, `highlightQuery`, `id`, `inputSize`, `keepMounted`, `keepMountedMode`, `leftSection`, `leftSectionPointerEvents`, `leftSectionProps`, `leftSectionWidth`, `limit`, `loading`, `loadingPosition`, `lockScroll`, `maxHeight`, `multiline`, `nothingFound`, `onEnterTransitionEnd`, `onExitTransitionEnd`, `onQueryChange`, `onSpotlightClose`, `onSpotlightOpen`, `overlayProps`, `padding`, `pointer`, `portalProps`, `query`, `radius`, `removeScrollProps`, `required`, `returnFocus`, `rightSection`, `rightSectionPointerEvents`, `rightSectionProps`, `rightSectionWidth`, `rootRef`, `scrollAreaComponent`, `scrollAreaProps`, `scrollable`, `searchProps`, `shadow`, `shortcut`, `stackId`, `store`, `success`, `tagsToIgnore`, `title`, `transitionProps`, `trapFocus`, `triggerOnContentEditable`, `withAria`, `withErrorStyles`, `withOverlay`, `withSuccessStyles`, `withinPortal`, `wrapperProps`, `xOffset`, `yOffset`, `zIndex`

### Carousel — X:Carousel

来源：[@mantine/carousel](https://mantine.dev/x/carousel/)；对应 `P10-04`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `controlSize`, `controlsOffset`, `emblaOptions`, `getEmblaApi`, `getIndicatorProps`, `height`, `includeGapInSize`, `initialSlide`, `nextControlIcon`, `nextControlProps`, `onNextSlide`, `onPreviousSlide`, `onSlideChange`, `orientation`, `plugins`, `previousControlIcon`, `previousControlProps`, `slideGap`, `slideSize`, `type`, `withControls`, `withIndicators`, `withKeyboardEvents`

**官方功能章节：** Installation；Do not forget to import styles；Documentation demos；Usage；Options；Embla options；Size and gap；Responsive styles；Container queries；Drag free；Vertical orientation；Controls icons；100% height；Get embla instance；Indicator styles；Hide inactive controls；Show controls on hover；Example: Images carousel；Example: Cards carousel；Accessibility

**本地实现：** [src/carousel/mod.rs](../src/carousel/mod.rs)、[src/carousel/model.rs](../src/carousel/model.rs)

**本地公开入口（同族）：** `autoplay`, `carousel`, `class`, `count`, `current`, `default_current`, `direction`, `enabled`, `focus`, `looping`, `name`, `node_ref`, `on`, `on_current_change`, `orientation`, `overscan`, `pause_on_hover`, `paused`, `propose`, `render_slide`, `size`, `style`, `swipe`, `theme`, `unstyled`, `validate`, `window`

**需要语义映射的属性候选：** `children`, `controlSize`, `controlsOffset`, `emblaOptions`, `getEmblaApi`, `getIndicatorProps`, `height`, `includeGapInSize`, `initialSlide`, `nextControlIcon`, `nextControlProps`, `onNextSlide`, `onPreviousSlide`, `onSlideChange`, `plugins`, `previousControlIcon`, `previousControlProps`, `slideGap`, `slideSize`, `type`, `withControls`, `withIndicators`, `withKeyboardEvents`

### Lightbox — X:Lightbox

来源：[@mantine/lightbox](https://mantine.dev/x/lightbox/)；对应 `P10-05`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `closeOnClickOutside`, `closeOnSwipeDown`, `currentIndex`, `emblaOptions`, `emblaRef`, `index`, `labels`, `loop`, `onClose`, `onIndexChange`, `opened`, `returnFocus`, `slide`, `slides`, `store`, `toolbarItems`, `transitionDuration`, `transitionProps`, `withDownload`, `withFullscreen`, `withInitialFocusPlaceholder`, `withKeyboardEvents`, `withNavigation`, `withSlideTransition`, `withThumbnails`, `withZoom`, `withinPortal`, `zIndex`, `zoomMaxScale`

**官方功能章节：** Installation；Usage；Slide data；Zoom；Thumbnails；All features；Loop navigation；Slide transition；Embla options；z-index；Open and close transition；Disable animations；Swipe to close；Click outside to close；Store API；Multiple lightboxes；Video slides；Custom slides；Custom toolbar；Compound components；Labels；Accessibility；Keyboard shortcuts

**本地实现：** [src/lightbox.rs](../src/lightbox.rs)

**本地公开入口（同族）：** `caption`, `class`, `default_open`, `default_view`, `enabled`, `factor`, `images`, `label`, `lightbox`, `lightbox_image`, `on_open_change`, `on_view_change`, `open`, `panel_ref`, `style`, `theme`, `title`, `trigger_ref`, `unstyled`, `view`

**需要语义映射的属性候选：** `children`, `closeOnClickOutside`, `closeOnSwipeDown`, `currentIndex`, `emblaOptions`, `emblaRef`, `index`, `labels`, `loop`, `onClose`, `onIndexChange`, `opened`, `returnFocus`, `slide`, `slides`, `store`, `toolbarItems`, `transitionDuration`, `transitionProps`, `withDownload`, `withFullscreen`, `withInitialFocusPlaceholder`, `withKeyboardEvents`, `withNavigation`, `withSlideTransition`, `withThumbnails`, `withZoom`, `withinPortal`, `zIndex`, `zoomMaxScale`

### Dropzone — X:Dropzone

来源：[@mantine/dropzone](https://mantine.dev/x/dropzone/)；对应 `P10-02`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `accept`, `acceptColor`, `activateOnClick`, `activateOnDrag`, `activateOnKeyboard`, `active`, `autoFocus`, `disabled`, `dragEventsBubbling`, `enablePointerEvents`, `getFilesFromEvent`, `inputProps`, `loaderProps`, `loading`, `maxFiles`, `maxSize`, `multiple`, `name`, `onDragEnter`, `onDragLeave`, `onDragOver`, `onDrop`, `onDropAny`, `onFileDialogCancel`, `onFileDialogOpen`, `onReject`, `openRef`, `portalProps`, `preventDropOnDocument`, `radius`, `rejectColor`, `useFsAccessApi`, `validator`, `withinPortal`, `zIndex`

**官方功能章节：** Installation；Usage；Dropzone.Accept, Dropzone.Reject and Dropzone.Idle；Loading state；Disabled state；Open file browser manually；Enable child pointer events；Mime types；Styles API；Images previews；Get ref；Dropzone.FullScreen component

**本地实现：** [../glory/crates/render/src/files.rs](../../glory/crates/render/src/files.rs)、[../glory/crates/web/src/browser/files.rs](../../glory/crates/web/src/browser/files.rs)、[src/files.rs](../src/files.rs)

**本地公开入口（同族）：** `attachment`, `class`, `clear`, `configuration`, `controller_ref`, `description`, `dropzone`, `enabled`, `file_attachment`, `file_button`, `file_input`, `files`, `files_ref`, `focus`, `label`, `matches`, `name`, `native_source`, `new`, `node_ref`, `on`, `on_files`, `on_result`, `preview`, `preview_in`, `previews`, `read`, `read_in`, `remove`, `size`, `status`, `status_ref`, `style`, `text`, `theme`, `unstyled`, `validate`, `validate_file`, `validate_files`, `validate_node`, `validate_read`, `validate_update`, `value`

**需要语义映射的属性候选：** `accept`, `acceptColor`, `activateOnClick`, `activateOnDrag`, `activateOnKeyboard`, `active`, `autoFocus`, `disabled`, `dragEventsBubbling`, `enablePointerEvents`, `getFilesFromEvent`, `inputProps`, `loaderProps`, `loading`, `maxFiles`, `maxSize`, `multiple`, `onDragEnter`, `onDragLeave`, `onDragOver`, `onDrop`, `onDropAny`, `onFileDialogCancel`, `onFileDialogOpen`, `onReject`, `openRef`, `portalProps`, `preventDropOnDocument`, `radius`, `rejectColor`, `useFsAccessApi`, `validator`, `withinPortal`, `zIndex`

### NavigationProgress — X:NavigationProgress

来源：[@mantine/nprogress](https://mantine.dev/x/nprogress/)；对应 `P5-09`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Installation；Setup NavigationProgress；Usage

**本地实现：** [src/navigation_progress.rs](../src/navigation_progress.rs)

**本地公开入口（同族）：** `cancel`, `class`, `finish`, `indeterminate`, `name`, `navigation_progress`, `node_ref`, `on_ready`, `set`, `snapshot`, `start`, `style`, `theme`, `thickness`, `unstyled`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。

### Modals manager — X:Modals manager

来源：[@mantine/modals](https://mantine.dev/x/modals/)；对应 `P5-13`；原生有限能力状态 `tested`。

**官方 Props / 子部件 Props：** `children`, `labels`, `modalProps`, `modals`

**官方功能章节：** Installation；Setup ModalsProvider；Confirm modal；Context modals；Typesafe context modals；Content modals；Multiple opened modals；Modal props；Dynamic content and the modals manager

**本地实现：** [../glory/crates/blitz/src/layers.rs](../../glory/crates/blitz/src/layers.rs)、[src/modals.rs](../src/modals.rs)、[src/overlay.rs](../src/overlay.rs)

**本地公开入口（同族）：** `allowed`, `ax_branch`, `boundary`, `cancel_label`, `capacity`, `class`, `close`, `close_all`, `close_content`, `close_label`, `confirm_enabled`, `confirm_label`, `confirm_variant`, `confirmation_request`, `content`, `content_policy`, `default_open`, `default_snap`, `description`, `dismiss_on_escape`, `dismiss_on_outside`, `enabled`, `initial_focus`, `label`, `modal`, `modal_request`, `modals_manager`, `name`, `on`, `on_cancel`, `on_confirm`, `on_dismiss`, `on_open_change`, `on_ready`, `on_result`, `on_snap_change`, `open`, `panel_ref`, `pending`, `placement`, `return_focus`, `show_close`, `show_trigger`, `side`, `size`, `snap_config`, `snap_index`, `snap_points`, `style`, `theme`, `title`, `trigger_content`, `trigger_ref`, `unstyled`, `update`

**需要语义映射的属性候选：** `children`, `labels`, `modalProps`, `modals`

### Rich text editor — X:Rich text editor

来源：[@mantine/tiptap](https://mantine.dev/x/tiptap/)；对应 `P10-08`；原生有限能力状态 `rejected`。

**官方 Props / 子部件 Props：** 该页没有独立 Props 表。

**官方功能章节：** Installation；TipTap editor；Usage；Subtle variant；Controlled；Controls and extensions；Placeholder；Link extension；Text color；Code highlight；Source code mode；Tasks；Image upload；Table；Details；Invisible characters；Removing the paragraph wrapper；Typography styles；Bubble menu；Floating menu；Sticky toolbar；Editor context；Custom controls；Change icons；Labels and localization；FORM COMPONENTS AND FEATURES

**本地实现：** [../glory/crates/render/src/rich.rs](../../glory/crates/render/src/rich.rs)、[../glory/crates/render/src/rich_document.rs](../../glory/crates/render/src/rich_document.rs)、[src/rich_editor.rs](../src/rich_editor.rs)

**本地公开入口（同族）：** `charged_bytes`, `class`, `collapsed`, `composing`, `controller_ref`, `default_document`, `document`, `enabled`, `focus`, `format`, `format_at`, `history`, `insert`, `insert_at`, `is_collapsed`, `name`, `new`, `node_ref`, `on`, `on_document_change`, `ordered`, `plain_text`, `readonly`, `redo`, `replace`, `reset`, `rich_text_editor`, `selection`, `size`, `style`, `theme`, `toolbar`, `undo`, `unstyled`, `utf16_len`, `validate`, `validate_selection`

**需要语义映射的属性候选：** 无直接拼写候选；仍不推断功能等价。
