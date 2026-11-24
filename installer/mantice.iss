; Inno Setup script of the Mantice installer (Inno Setup 6.3 or later).
;
; Build it with installer\build.ps1, which reads the version from Cargo.toml. By hand:
;   ISCC.exe /DAppVersion=0.5.1 installer\mantice.iss
; Optional defines: /DExeDir=<folder of mantice.exe> (default: target\release) and
; /DIconFile=<.ico file> (the icon of the setup program, default: the Inno Setup one).
;
; The installer is per-user by default (no administrator rights, no UAC prompt): the program lives in
; %LOCALAPPDATA%\Programs\Mantice. The dialog at the start lets the user install for all users instead.
; The data of the user (%LOCALAPPDATA%\Mantice: combos, settings, backups) is never touched, not even by
; the uninstaller. The program itself moves there the folder of ExpanderCraft (the former name) at its first launch.
;
; Mantice replaces an installed ExpanderCraft: the AppId is the same, so Setup reuses its folder, and the files
; of the old name (the executable, the shortcuts, the startup entry) are removed by [InstallDelete] and [Registry].

#ifndef AppVersion
  #error AppVersion is not defined: pass /DAppVersion=x.y.z (installer\build.ps1 reads it from Cargo.toml)
#endif
#ifndef ExeDir
  #define ExeDir "..\target\release"
#endif

#define AppName "Mantice"
#define AppExe "mantice.exe"
#define AppUrl "https://github.com/echidistanti/mantice"

[Setup]
; Never change the AppId: it is what makes a new version replace the previous one.
AppId={{9F76FD54-68FA-4B68-998C-BDA0B4BE0160}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=echidistanti
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}/issues
AppUpdatesURL={#AppUrl}/releases
VersionInfoVersion={#AppVersion}.0
VersionInfoProductName={#AppName}
VersionInfoDescription={#AppName} setup

DefaultDirName={autopf}\{#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0

; Mantice keeps this mutex while it runs, in the tray too: Setup and the uninstaller ask the user to quit
; it instead of trying to replace a program in use. The first name is MUTEX_NAME in src/platform/windows.rs, the
; second one is the mutex of ExpanderCraft, so that an old version still running is detected too.
AppMutex=Local\Mantice.SingleInstance.6f1c2a4e,Local\ExpanderCraft.SingleInstance.6f1c2a4e
SetupMutex=ManticeSetup

UninstallDisplayIcon={app}\{#AppExe}
OutputDir=Output
OutputBaseFilename=Mantice-{#AppVersion}-windows-x64-setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
#ifdef IconFile
SetupIconFile={#IconFile}
#endif

; The same languages as the program. Setup picks the one of Windows, English when it is not among these.
[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#ExeDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[InstallDelete]
; What an installed ExpanderCraft leaves behind: the executable and the shortcuts of the old name.
Type: files; Name: "{app}\expandercraft.exe"
Type: files; Name: "{autoprograms}\ExpanderCraft.lnk"
Type: files; Name: "{autodesktop}\ExpanderCraft.lnk"

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; "Launch Mantice at login" (a preference of the program) writes this value: remove it on uninstall,
; so that Windows does not look for a program that is gone.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "{#AppName}"; ValueType: none; Flags: dontcreatekey uninsdeletevalue
; The value of ExpanderCraft points to the executable that [InstallDelete] removes: the program writes its own
; again at the next launch when "Launch at login" is on.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "ExpanderCraft"; ValueType: none; Flags: dontcreatekey deletevalue

[Run]
; runasoriginaluser: with an all-users install Setup is elevated, but the program has to run with the rights of
; the user (an elevated process would only be able to type into the windows of other elevated programs).
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent runasoriginaluser
