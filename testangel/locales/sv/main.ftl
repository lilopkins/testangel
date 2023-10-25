app-name = TestAngel

# Generic useful words and phrases
ok = Okej
save = Spara
discard = Kasta
value = Värde
nothing-open = Inget är öppet

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
flow-filetype = { app-name } flödefil
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

# Execution

report-failed = Det gick inte att generera rapport
report-failed-message = Det gick inte att generera rapporten: { $reason }
report-save-title = Spara rapport...
report-default-name = rapport.pdf
pdf-files = PDF-filer
