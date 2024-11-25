; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
MinVersion=6.1.7600
AppId=14a2561f-0757-4737-96c9-79ddfd482c90
AppName=test_c
AppVersion=2024.11.25
DefaultDirName={autopf64}\test_c
UsePreviousAppDir=no
DisableDirPage=no
DefaultGroupName=test_c
AllowNoIcons=yes
DisableFinishedPage=yes
;PrivilegesRequired=lowest
PrivilegesRequired=admin
OutputDir=.\target
OutputBaseFilename=test_c
;SetupIconFile=setup.ico
Uninstallable=yes
;UninstallDisplayIcon=setup.ico
UninstallDisplayName=test_c
CreateUninstallRegKey=no
Compression=lzma/ultra64   
SolidCompression=yes
ArchitecturesAllowed=x64os arm64
ArchitecturesInstallIn64BitMode=x64os arm64
WizardStyle=modern
AlwaysRestart=no
RestartIfNeededByRun=no
VersionInfoProductTextVersion="v2024.11.25"
VersionInfoProductVersion="2024.11.25.0"
VersionInfoTextVersion="v2024.11.25"
VersionInfoVersion="2024.11.25.0"
VersionInfoProductName="test_c - Package 2024.11.25"


[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}";


[Files]
Source: "target\installed\x64-windows\bin\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs;
; NOTE: Don't use "Flags: ignoreversion" on any shared system files


[Icons]
Name: "{group}\test_c"; Filename: "{app}\test_c.exe"
Name: "{group}\{cm:UninstallProgram,test_c}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\test_c"; Filename: "{app}\test_c.exe"; Tasks: desktopicon


[InstallDelete]
Type: filesandordirs; Name: "{app}"


[UninstallDelete]
Type: filesandordirs; Name: "{app}"