app-name = TestAngel

# Generic useful words and phrases
ok = Ok
save = Save
discard = Discard
value = Value
nothing-open = Nothing is Open
delete = Delete

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
source-from-param = From Parameter: { $param }

# File types

filetype-all = All files
filetype-flow = { app-name } Flow file
filetype-action = { app-name } Action file
filetype-pdf = PDF files

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
flow-nothing-open-description = Open a flow or add a step to get started

flow-action-changed = Flow Actions Changed
flow-action-changed-message = The parameters in { $stepCount ->
        [one] step
       *[other] steps
    } { $steps } have changed so they has been reset.

flow-save-before = Save this Flow?
flow-save-before-message = This flow has been modified since it was last saved. Would you like to save it before continuing?
flow-saved = Flow saved
flow-error-saving = Error saving flow
flow-error-opening = Error opening flow

flow-save-open-error-io-error = I/O error: { $error }
flow-save-open-error-parsing-error = The flow file is corrupted: { $error }
flow-save-open-error-serializing-error = The flow could not be saved due to an internal serialisation error: { $error }
flow-save-open-error-flow-not-version-compatible = The flow you tried to load is not compatible with this version of { app-name }.
flow-save-open-error-missing-action = The action for step { $step } (with internal ID: { $error }) in this flow is missing.

flow-execution-running = Flow running...
flow-execution-failed = Flow failed.
flow-execution-failed-message = Flow failed at step { $step }: { $reason }

flow-step-label = Step { $step }: { $name }

# Actions

action-header-add = Add step
action-header-run = Test action
action-header-more = More...
action-header-new = New action
action-header-open = Open action...
action-header-save = Save action
action-header-save-as = Save action as...
action-header-close = Close action
action-header-about = About { app-name }
action-nothing-open-description = Open an action or add a step to get started

action-save-before = Save this action?
action-save-before-message = This action has been modified since it was last saved. Would you like to save it before continuing?
action-saved = Action saved
action-error-saving = Error saving action
action-error-opening = Error opening action

action-save-open-error-io-error = I/O error: { $error }
action-save-open-error-parsing-error = The action file is corrupted: { $error }
action-save-open-error-serializing-error = The action could not be saved due to an internal serialisation error: { $error }
action-save-open-error-action-not-version-compatible = The action you tried to load is not compatible with this version of { app-name }.
action-save-open-error-missing-instruction = The instruction for step { $step } (with internal ID: { $error }) in this action is missing.

action-metadata-label = Action Metadata
action-metadata-name = Action Name
action-metadata-group = Action Group
action-metadata-author = Author
action-metadata-description = Description
action-metadata-visible = Visible in Flow Editor

action-params-new = New Input Parameter
action-outputs-new = New Output

action-step-set-comment = Set step comment
action-step-label = Step { $step }: { $name }
action-condition-run-always = Always runs
action-condition-run-condition = Runs if: { $cond }

# Execution

report-failed = Failed to produce report
report-failed-message = Failed to produce: { $reason }
report-save-title = Save evidence...
report-default-name = report.pdf
