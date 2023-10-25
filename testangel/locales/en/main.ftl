app-name = TestAngel

# Generic useful words and phrases
ok = Ok
save = Save
discard = Discard
value = Value
nothing-open = Nothing is Open

acknowledgements-testing-title = Software Testing
acknowledgements-translations-title = Translations

tab-flows = Flows
tab-actions = Actions

variable-row-edit-param = Edit Parameter
variable-row-subtitle = { $kind }, { $source }
variable-row-subtitle-with-value = { $kind }, { $source }: { $value }

drag-drop-here = Drop step here
move-up = Move up
move-down = Move down
delete-step = Delete step

source-literal = Literal value
source-from-step = From Step { $step }: { $name }

# Flows

flow-header-add = Add step
flow-header-run = Run flow
flow-header-more = More...
flow-header-new = New flow
flow-header-open = Open flow...
flow-header-save = Save flow
flow-header-save-as = Save flow as...
flow-header-close = Close flow
flow-header-about = About { app-name }
flow-filetype = { app-name } Flow file
flow-nothing-open-description = Open a flow or add a step to get started

flow-action-changed = Flow Actions Changed
flow-action-changed-message = The parameters in steps { $steps } have changed so it has been reset.

flow-save-before = Save this Flow?
flow-save-before-message = This flow has been modified since it was last saved. Would you like to save it before continuing?
flow-saved = Flow saved.
flow-error-saving = Error saving flow
flow-error-opening = Error Opening Flow

flow-save-open-error-io-error = I/O error: { $error }
flow-save-open-error-parsing-error = The flow file is corrupted: { $error }
flow-save-open-error-serializing-error = The flow could not be saved due to an internal serialisation error: { $error }
flow-save-open-error-flow-not-version-compatible = The flow you tried to load is not compatible with this version of { app-name }.
flow-save-open-error-missing-action = The action for step { $step } (with internal ID: { $error }) in this flow is missing.

flow-execution-running = Flow running...
flow-execution-failed = Flow failed.
flow-execution-failed-message = Flow failed at step { $step }: { $reason }

flow-step-label = Step { $step }: { $name }

# Execution

report-failed = Failed to produce report
report-failed-message = Failed to produce: { $reason }
report-save-title = Save evidence...
report-default-name = report.pdf
pdf-files = PDF files
