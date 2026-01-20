; Praymodoro Inno Setup Script
; Creates a professional Windows installer

#define MyAppName "Praymodoro"
#define MyAppVersion GetEnv('VERSION')
#define MyAppPublisher "John VonDrashek"
#define MyAppURL "https://github.com/JohnVonDrashek/praymodoro"
#define MyAppExeName "praymodoro.exe"

[Setup]
; Unique app ID
AppId={{7B8E4F2A-3C9D-4E5F-8A1B-2C6D9E0F3A4B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Output settings
OutputDir=..\target\release
OutputBaseFilename=Praymodoro_{#MyAppVersion}_setup
; Compression
Compression=lzma2/ultra64
SolidCompression=yes
; Installer appearance
SetupIconFile=..\assets\icons\Praymodoro.ico
WizardStyle=modern
WizardSizePercent=100
; Privileges - install for current user only (no admin required)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
; Uninstaller
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startupicon"; Description: "Start {#MyAppName} when Windows starts"; GroupDescription: "Startup:"; Flags: unchecked

[Files]
; Main executable
Source: "..\target\release\praymodoro.exe"; DestDir: "{app}"; Flags: ignoreversion
; Icon
Source: "..\assets\icons\Praymodoro.ico"; DestDir: "{app}"; Flags: ignoreversion
; Assets
Source: "..\assets\characters\*"; DestDir: "{app}\assets\characters"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\assets\ui\*"; DestDir: "{app}\assets\ui"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\assets\fonts\*"; DestDir: "{app}\assets\fonts"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\Praymodoro.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\Praymodoro.ico"; Tasks: desktopicon

[Registry]
; Auto-start on Windows login (if selected)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: startupicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
