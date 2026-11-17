; Installatieprogramma voor Jaap's Programmeertaal (Inno Setup 6).
; Wordt gemaakt door .github/workflows/bouwen.yml: ISCC /DVersie=0.1.0 installer\JaapsProgrammeertaal.iss
; De map installatiemap bevat: JaapsProgrammeertaal.exe, jaaptaal.exe, Handleiding.txt, LEES MIJ.txt,
; Wijzigingen.txt, de map voorbeelden en de map rust (de Rust-compiler, zodat je niets apart hoeft te installeren).
;
; Installeren mag met en zonder beheerdersrechten. Standaard: alleen voor jou, zonder beheerdersrechten
; (in %LOCALAPPDATA%\Programs). Bij de start vraagt het installatieprogramma of het voor alle gebruikers mag;
; dan is wel een beheerder nodig. Op de opdrachtregel: /CURRENTUSER of /ALLUSERS.

#ifndef Versie
  #define Versie "0.1.0"
#endif

[Setup]
AppId={{9C3E5B71-2A4D-4E8F-B1C6-7D0A3F5E2B94}
AppName=Jaap's Programmeertaal
AppVersion={#Versie}
AppVerName=Jaap's Programmeertaal {#Versie}
AppPublisher=Jaap Wortel
VersionInfoVersion={#Versie}
DefaultDirName={autopf}\Jaaps Programmeertaal
DefaultGroupName=Jaap's Programmeertaal
DisableProgramGroupPage=yes
; Zo min mogelijk schermen (zoals in Jaaps andere programma's).
DisableWelcomePage=yes
DisableDirPage=yes
DisableReadyPage=yes
DisableReadyMemo=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..
OutputBaseFilename=Jaaps Programmeertaal {#Versie} installatie
UninstallDisplayIcon={app}\JaapsProgrammeertaal.exe
UninstallDisplayName=Jaap's Programmeertaal
Compression=lzma2/max
SolidCompression=yes
LZMAUseSeparateProcess=yes
WizardStyle=modern
CloseApplications=yes

[Languages]
Name: "nl"; MessagesFile: "compiler:Languages\Dutch.isl"

[Tasks]
; Zonder kopje erboven: dan leest de schermlezer het vakje gewoon voor.
Name: "bureaublad"; Description: "Een snelkoppeling op het bureaublad zetten"

[InstallDelete]
; Een nieuwe versie: de oude Rust-compiler eerst helemaal weg, zodat er geen oude onderdelen blijven staan.
Type: filesandordirs; Name: "{app}\rust"

[Files]
Source: "..\installatiemap\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{autoprograms}\Jaap's Programmeertaal"; Filename: "{app}\JaapsProgrammeertaal.exe"
Name: "{autoprograms}\Handleiding van Jaap's Programmeertaal"; Filename: "{app}\Handleiding.txt"
Name: "{autodesktop}\Jaap's Programmeertaal"; Filename: "{app}\JaapsProgrammeertaal.exe"; Tasks: bureaublad

[Run]
Filename: "{app}\JaapsProgrammeertaal.exe"; Description: "Jaap's Programmeertaal nu starten"; Flags: nowait postinstall skipifsilent
