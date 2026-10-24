# Questionnaire

Enable `chat`. `questionnaire(key).step(question_step(key, label).question(...))`
builds a real Form, finite Stepper, stable step Fieldsets and typed questions.
`question(key, Cage<T>, FieldOptions<T>, render)` retains the original `T`; it
requires neither `Clone`, JSON nor a template/schema VM. The render factory
receives `FormField<T>` and `FieldControl`. Pass the context into the actual
control with `.field(control)`, accept application proposals explicitly and call
`field.changed()` after an accepted user edit. The factory runs once in its
mounted Owner. Inactive steps remain mounted and their real Fieldsets are disabled.

Next validates the current step through `FormState`, including async validators,
before proposing the next step. Final submission freshly validates every typed
field, including answers in earlier hidden steps. An earlier failure proposes
that step and exposes its Field error. `.active(source).on_step_change(...)` is
controlled: refusing a proposal leaves the current panel and answers accepted.
Back cancels validation/submission; it preserves answers. A new accepted step,
changed answer or retiring Owner revokes obsolete validation and completion.
Late recovery actions recheck their current numeric attempt, remaining error,
accepted step, live/enabled ancestry and submission status.

`on_complete(SubmitToken)` runs in an Owner task after validation and the complete
host tick. The application reads its typed answer Cages and owns authorization,
transport, token completion and routing. Success does not clear answers, unmount
the form or navigate automatically. Editing/back can invalidate an outstanding
token; late `complete` returns false. Native FormData contains controls from the
current enabled step; the complete questionnaire answer is the application's
typed model, rather than a hidden collection of successful inputs. Optional
`FieldOptions::serialize` encoders retain the ordinary FormState contract.

The schema admits 1–16 nonempty steps, at most 32 questions per step and 256 in
total. Keys are unique within their schema scope and at most 64 ASCII bytes.
Labels are nonblank canonical single-line text up to 256 bytes; descriptions are
at most 4,096 bytes. Iterator intake stops at the first extra step. Configuration,
foreign/retired answer handles and all declaration budgets are validated before
calling any question factory. Arbitrary factory contents and business validation
cost remain the application's responsibility; the registry does not scan a
business answer source or keep visited questions.

Evidence: `D:/BuildCache/sapid/chat-actions-tests.log` includes four Questionnaire
regressions: typed String/bool validation and fresh final validation, controlled
step refusal, million-step bounded intake/duplicate schema rejection, async Back
and Owner cancellation. The repeated-invalid baseline was reproduced in
`chat-questionnaire-repeated-invalid-baseline.log`: two invalid outcomes could
coalesce through Validating without resetting a boolean recovery marker. A
non-wrapping current-attempt number and late validation checks now cover it.
`chat-actions-native-tests.log` includes the four-theme real native form test:
actual geometry, hidden/disabled controls, accepted edits, checkbox, successful
callback, editor engine identity/ASCII selection retention and raster output.
`chat-actions-web-strict.log` covers the caller. Immutable FORMAT13 Web43 passes
all twelve original cases with no page/console errors
(`questionnaire-browser-web43.json`, Chromium 148; script SHA256
`846e57849166a3e3dc6e2c458b1694646661d643e692361076553dd51cd29a88`).
This includes four themes in CSR/adoption, current-step required errors, successive
final validation failures, retained earlier editor/later bool answer, controlled
step refusal/back, mobile geometry, actual current-step AX and Owner remount.
The original Web42 run caught the repeated-invalid fault; its failure logs remain.
The first selector was also corrected from the inline check caption to the actual
Field-provided accessible name, `Permission`.
