; Slice 4D Shooter - the first multiplayer shooter set in 4D space
; Copyright (C) 2023-2026  Timofei Molokov

[Setup]
AppName=Slice 4D Shooter
; The value is patched from CI by the "Patch installer.iss appVersion" step
AppVersion=0.0.0
DefaultDirName={pf}\Slice 4D Shooter
DefaultGroupName=Slice 4D Shooter
OutputBaseFilename=Slice4DShooterInstaller
Compression=lzma2
SolidCompression=yes
LicenseFile=LICENSE.txt
CloseApplications=yes
RestartApplications=no
ArchitecturesInstallIn64BitMode=x64
SetupIconFile=media\icon.ico
UninstallDisplayIcon={app}\Slice 4D Shooter.exe

[Files]
Source: "target\release\Slice 4D Shooter.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "media\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "dll\WinSparkle.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "CREDITS.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "dll\LICENSE.txt"; DestDir: "{app}\licenses"; DestName: "WinSparkle-LICENSE.txt"; Flags: ignoreversion
Source: "matchbox_signaling_modified\LICENSE-MIT"; DestDir: "{app}\licenses"; DestName: "Matchbox-LICENSE-MIT.txt"; Flags: ignoreversion
Source: "matchbox_signaling_modified\LICENSE-APACHE"; DestDir: "{app}\licenses"; DestName: "Matchbox-LICENSE-APACHE.txt"; Flags: ignoreversion
Source: "matchbox_signaling_modified\NOTICE"; DestDir: "{app}\licenses"; DestName: "Matchbox-NOTICE.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\Slice 4D Shooter"; Filename: "{app}\Slice 4D Shooter.exe"; IconFilename: "{app}\icon.ico"
Name: "{userdesktop}\Slice 4D Shooter"; Filename: "{app}\Slice 4D Shooter.exe"; Tasks: desktopicon; IconFilename: "{app}\icon.ico"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional tasks"

[Run]
Filename: "{app}\Slice 4D Shooter.exe"; Description: "Launch Slice 4D Shooter"; Flags: nowait postinstall skipifsilent
