use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::output_form(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    use sapid::chat::*;
    use std::{cell::Cell, rc::Rc};
    pub fn output_form(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let first = scope.cage(1000u64);
            let count = scope.cage(1_000_000usize);
            let revision = scope.cage(0u64);
            let expanded = scope.cage(false);
            let mounted = scope.cage(true);
            let info = scope.cage(String::new());
            let reads = Rc::new(Cell::new(0usize));
            let counts = scope.cage(TestCounts {
                passed: 1,
                failed: 1,
                skipped: 1,
                running: 1,
            });
            let root = scope.cage(None);
            view()
                .class("output-fixture")
                .style(
                    Style::new()
                        .display(glory::style::Display::Flex)
                        .flex_direction(glory::style::FlexDirection::Column)
                        .gap(12.)
                        .min_width(Length::Px(0.)),
                )
                .node_ref(root)
                .show_in(scope)?;
            let area = scope.child_in(root.get().unwrap())?;
            for (label, action) in [
                ("Append five log records", 0),
                ("Prepend ten log records", 1),
                ("Change output heights", 2),
                ("Use billion records", 3),
                ("Toggle output owner", 4),
                ("Clear test results", 5),
                ("Restore test results", 6),
            ] {
                sapid::button()
                    .theme(theme.clone())
                    .label(label)
                    .on_press(move || match action {
                        0 => {
                            if count.get_untracked() <= MAX_OUTPUT_RECORDS - 5 {
                                count.revise(|n| *n += 5);
                                revision.revise(|n| *n += 1);
                            }
                        }
                        1 => {
                            if first.get_untracked() >= 10
                                && count.get_untracked() <= MAX_OUTPUT_RECORDS - 10
                            {
                                first.revise(|n| *n -= 10);
                                count.revise(|n| *n += 10);
                                revision.revise(|n| *n += 1);
                            }
                        }
                        2 => expanded.revise(|v| *v = !*v),
                        3 => {
                            count.revise(|n| *n = MAX_OUTPUT_RECORDS);
                            revision.revise(|n| *n += 1);
                        }
                        4 => mounted.revise(|v| *v = !*v),
                        5 => counts.revise(|n| *n = TestCounts::default()),
                        _ => counts.revise(|n| {
                            *n = TestCounts {
                                passed: 1,
                                failed: 1,
                                skipped: 1,
                                running: 1,
                            }
                        }),
                    })
                    .show_in(&area)?;
            }
            let palette = theme.clone();
            glory::ui::when(mounted,move||{let theme=palette.clone();let reads=reads.clone();widget(move|scope|{
   let sampled=reads.clone();let report=reads.clone();
   terminal("fixture-terminal",count,move|i|first.get()+i as u64,move|key|key.checked_sub(first.get()).and_then(|n|usize::try_from(n).ok()).filter(|i|*i<count.get()),move|i|{
    sampled.set(sampled.get()+1);let key=first.get()+i as u64;
    LogRecord{key,stream:if key.is_multiple_of(3){LogStream::Stderr}else{LogStream::Stdout},level:if key.is_multiple_of(3){LogLevel::Error}else{LogLevel::Info},payload:if key.is_multiple_of(7){LogPayload::Redacted}else{LogPayload::Text(format!("Record {key}: literal <script>window.outputExecuted=true</script>\u{1b}[31m{}",if expanded.get(){"\nAdditional application output\nA taller accepted row"}else{""}))}}
   }).theme(theme.clone()).class("terminal-primary").height(260.).estimate(70.).row_limit(32).measure_limit(64).revision(revision).inspect(move|s|{
    let next=format!("range {}..{}; rows {}; slots {}; measured {}; reads {}; following window {}",s.range.start,s.range.end,s.rows,s.height_slots,s.measured,report.get(),s.logical);
    if info.get_untracked()!=next{info.revise(|v|*v=next);}
   }).show_in(scope)?;
   test_results("fixture-tests",counts,|i|i as u64,move|k|usize::try_from(k).ok().filter(|i|*i<counts.with(|c|c.total().unwrap())),|i|TestCase{key:i as u64,name:["parser preserves UTF-16","renderer applies accepted prefix","unix AT client acceptance","native raster comparison"][i].into(),status:[TestStatus::Passed,TestStatus::Failed,TestStatus::Skipped,TestStatus::Running][i],elapsed_millis:if i<2{Some(17+i as u64)}else{None},output:if i==2{LogPayload::Redacted}else{LogPayload::Text("Application-supplied test output\n<script>window.testExecuted=true</script>\u{1b}[31m".into())}}).theme(theme.clone()).class("test-results-primary").height(250.).initially_following(false).show_in(scope)?;
   Ok(())
  })}).show_in(&area)?;
            sapid::text()
                .theme(theme.clone())
                .class("output-stats")
                .content(info)
                .show_in(&area)?;
            sapid::text()
                .theme(theme.clone())
                .class("execution-count")
                .content("Commands executed by this display: 0")
                .show_in(&area)?;
            scope.retain(area)?;
            Ok(())
        })
    }
}
