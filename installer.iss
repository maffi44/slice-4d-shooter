; Slice 4D Shooter - the first multiplayer shooter set in 4D space
; Copyright (C) 2023-2025  Timofei Molokov

; This program is free software: you can redistribute it and/or modify
; it under the terms of the GNU Affero General Public License as
; published by the Free Software Foundation, either version 3 of the
; License, or (at your option) any later version.

; This program is distributed in the hope that it will be useful,
; but WITHOUT ANY WARRANTY; without even the implied warranty of
; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
; GNU Affero General Public License for more details.

; You should have received a copy of the GNU Affero General Public License
; along with this program.  If not, see <https://www.gnu.org/licenses/>.

[Setup]
AppName=Slice 4D Shooter
; The value is patched from CI by the "Patch installer.iss appVersion" step
AppVersion=0.0.0
DefaultDirName={pf}\Slice 4D Shooter
DefaultGroupName=Slice 4D Shooter
OutputBaseFilename=Slice 4D Shooter Installer
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

[Icons]
Name: "{group}\Slice 4D Shooter"; Filename: "{app}\Slice 4D Shooter.exe"; IconFilename: "{app}\icon.ico"
Name: "{userdesktop}\Slice 4D Shooter"; Filename: "{app}\Slice 4D Shooter.exe"; Tasks: desktopicon; IconFilename: "{app}\icon.ico"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional tasks"

[Run]
Filename: "{app}\Slice 4D Shooter.exe"; Description: "Launch Slice 4D Shooter"; Flags: nowait postinstall skipifsilent
