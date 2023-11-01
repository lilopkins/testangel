app-name = TestAngel

# Generic useful words and phrases
ok = Okej
save = Spara
discard = Kasta
value = Värde
nothing-open = Inget är öppet
delete = Ta bort

acknowledgements-testing-title = Programtestning
acknowledgements-translations-title = Översättningar

tab-flows = Flöder
tab-actions = Åtgärder

variable-row-edit-param = Redigera parameter
variable-row-subtitle = { $kind }, { $source }
variable-row-subtitle-with-value = { $kind }, { $source }: { $value }

drag-drop-here = Dra och släpp hit
move-up = Flytta upp
move-down = Flytta ned
delete-step = Ta bort steg

source-literal = Direktvärde
source-from-step = Från steg { $step }: { $name }
source-from-param = From Parameter: { $param }

# File types

filetype-all = Alla filer
filetype-flow = { app-name } flödefil
filetype-action = { app-name } Action file
filetype-pdf = PDF-filer

# Flows

flow-header-add = Lägg till steg
flow-header-run = Kör flöde
flow-header-more = Mer...
flow-header-new = Nytt flöde
flow-header-open = Öppna flöde...
flow-header-save = Spara flöde
flow-header-save-as = Spara flöde som...
flow-header-close = Stäng flöde
flow-header-about = Om { app-name }
flow-nothing-open-description = Öppna en flöde eller lägg till ett steg för att komma igång

flow-action-changed = Åtgärder i flödet har ändrats
flow-action-changed-message = Parametrarna i steg { $steps } har ändrats, vilket har återställt dem.

flow-save-before = Spara detta flöde?
flow-save-before-message = Det här flödet har ändrats sedan det senast sparades. Vill du spara det innan du fortsätter?
flow-saved = Flödet har sparats.
flow-error-saving = Fel vid sparning av flödet.
flow-error-opening = Fel vid öppning av flödet.

flow-save-open-error-io-error = I/O fel: { $error }
flow-save-open-error-parsing-error = Felaktig flödefil: { $error }
flow-save-open-error-serializing-error = Det går inte att spara flödet på grund av ett internt fel i serialiseringen: { $error }
flow-save-open-error-flow-not-version-compatible = Flödet du har öppnat är inte kompatibelt med den här versionen av { app-name }.
flow-save-open-error-missing-action = Åtgärden för steg { $step } (med internt identifierare: { $error }) i detta flöde saknas.

flow-execution-running = Flöde körs...
flow-execution-failed = Flödet misslyckades.
flow-execution-failed-message = Flödet misslyckades på steg { $step }: { $reason }

flow-step-label = Steg { $step }: { $name }

# Actions

action-header-add = Lägg till steg
action-header-more = Mer...
action-header-new = Ny åtgärd
action-header-open = Öppna åtgärd...
action-header-save = Spara åtgärd
action-header-save-as = Spara åtgärd som...
action-header-close = Stäng åtgärd
action-header-about = Om { app-name }
action-nothing-open-description = Öppna en åtgärd eller lägg till ett steg för att komma igång

action-save-before = Spara denna åtgärden?
action-save-before-message = Det här åtgärden har ändrats sedan det senast sparades. Vill du spara det innan du fortsätter?
action-saved = Åtgärden har sparats.
action-error-saving = Fel vid sparning av åtgärden.
action-error-opening = Fel vid öppning av åtgärden.

action-save-open-error-io-error = I/O fel: { $error }
action-save-open-error-parsing-error = Felaktig åtgärdsfil: { $error }
action-save-open-error-serializing-error = Det går inte att spara åtgärden på grund av ett internt fel i serialiseringen: { $error }
action-save-open-error-action-not-version-compatible = Åtgärden du har öppnat är inte kompatibelt med den här versionen av { app-name }.
action-save-open-error-missing-instruction = Instruktionen för steg { $step } (med internt identifierare: { $error }) i denna åtgärden saknas.

action-metadata-label = Åtgärdsdata
action-metadata-name = Åtgördsnamn
action-metadata-group = Åtgärdsgrupp
action-metadata-author = Skapare
action-metadata-description = Beskrivning
action-metadata-visible = Visible in Flow Editor

action-params-new = Ny ingångsparameter
action-params-name-placeholder = Parametersnamn
action-outputs-new = Ny utgång
action-outputs-name-placeholder = Utgångsnamn

action-step-set-comment = Sätt steg kommentar
action-step-label = Steg { $step }: { $name }
action-condition-run-always = Kör alltid
action-condition-run-condition = Kör ifall: { $cond }

# Execution

report-failed = Det gick inte att generera rapport
report-failed-message = Det gick inte att generera rapporten: { $reason }
report-save-title = Spara rapport...
report-default-name = rapport.pdf
