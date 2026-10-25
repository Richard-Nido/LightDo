Unicode true
!include "MUI2.nsh"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!ifndef VERSION
!error "VERSION is required"
!endif
!ifndef STAGE
!error "STAGE is required"
!endif
!ifndef OUTPUT
!error "OUTPUT is required"
!endif
Name "轻待 LightDo"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\LightDo"
InstallDirRegKey HKCU "Software\LightDo" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "LightDo · Yuan Haoran (Richard-Nido)"
Icon "..\assets\lightdo.ico"
UninstallIcon "..\assets\lightdo.ico"
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "LightDo"
VIAddVersionKey "CompanyName" "Yuan Haoran (Richard-Nido)"
VIAddVersionKey "FileDescription" "轻待 LightDo 安装程序"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Copyright (c) 2026 Yuan Haoran"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\lightdo.exe"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
UninstPage custom un.DataPage un.DataLeave
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
Var DeleteData
Var DeleteDataBox

Function .onInit
 SetShellVarContext current
 SetRegView 64
 ${IfNot} ${RunningX64}
  MessageBox MB_OK|MB_ICONSTOP "轻待仅支持 Windows 11 x64。"
  Abort
 ${EndIf}
 ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\Windows NT\CurrentVersion" "CurrentBuildNumber"
 ; Build number is stored as a string by Windows.
 ReadRegStr $0 HKLM "SOFTWARE\Microsoft\Windows NT\CurrentVersion" "CurrentBuildNumber"
 ${If} $0 < 22000
  MessageBox MB_OK|MB_ICONSTOP "需要 Windows 11 x64（22000 或更新）。"
  Abort
 ${EndIf}
 Call EnsureClosed
FunctionEnd

Function EnsureClosed
 retry:
 System::Call 'kernel32::OpenMutexW(i 0x100000, i 0, w "Local\LightDo.SingleInstance") p.r0'
 ${If} $0 != 0
  System::Call 'kernel32::CloseHandle(p r0)'
  IfSilent silent_error
  MessageBox MB_RETRYCANCEL|MB_ICONINFORMATION "轻待正在运行。请先在托盘菜单中选择“退出轻待”，保存完成后再点重试。安装器不会强制结束进程。" IDRETRY retry
  Abort
  silent_error:
  SetErrorLevel 1618
  Abort
 ${EndIf}
FunctionEnd

Section "轻待（必需）" Main
 SectionIn RO
 SetOutPath "$INSTDIR"
 File "${STAGE}\lightdo.exe"
 File "${STAGE}\LICENSE"
 File "${STAGE}\THIRD_PARTY_NOTICES.md"
 File "${STAGE}\THIRD_PARTY_DEPENDENCIES.txt"
 SetOutPath "$INSTDIR\licenses"
 File "${STAGE}\LICENSE-Glance"
 File "${STAGE}\LICENSE-QuietDo"
 File "${STAGE}\licenses\*"
 WriteUninstaller "$INSTDIR\uninstall.exe"
 WriteRegStr HKCU "Software\LightDo" "InstallDir" "$INSTDIR"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "DisplayName" "轻待 LightDo"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "DisplayVersion" "${VERSION}"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "Publisher" "Yuan Haoran (Richard-Nido)"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "InstallLocation" "$INSTDIR"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "DisplayIcon" "$INSTDIR\lightdo.exe"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "UninstallString" '"$INSTDIR\uninstall.exe"'
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "URLInfoAbout" "https://github.com/Richard-Nido/LightDo"
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "NoModify" 1
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo" "NoRepair" 1
 CreateDirectory "$SMPROGRAMS\LightDo"
 CreateShortcut "$SMPROGRAMS\LightDo\轻待 LightDo.lnk" "$INSTDIR\lightdo.exe"
 CreateShortcut "$SMPROGRAMS\LightDo\卸载轻待.lnk" "$INSTDIR\uninstall.exe"
SectionEnd
Section /o "桌面快捷方式" DesktopShortcut
 CreateShortcut "$DESKTOP\轻待 LightDo.lnk" "$INSTDIR\lightdo.exe"
SectionEnd

Function un.onInit
 SetShellVarContext current
 SetRegView 64
 System::Call 'kernel32::OpenMutexW(i 0x100000, i 0, w "Local\LightDo.SingleInstance") p.r0'
 ${If} $0 != 0
  System::Call 'kernel32::CloseHandle(p r0)'
  IfSilent +2
  MessageBox MB_OK|MB_ICONINFORMATION "请先在托盘退出轻待，再卸载。不会强制结束进程。"
  SetErrorLevel 1618
  Abort
 ${EndIf}
 StrCpy $DeleteData 0
FunctionEnd
Function un.DataPage
 nsDialogs::Create 1018
 Pop $0
 ${NSD_CreateLabel} 0 0 100% 34u "默认保留任务和设置，重新安装后可继续使用。仅当勾选下方选项时删除 LightDo 自己的数据目录；不会影响 QuietDo 或 Glance。"
 Pop $0
 ${NSD_CreateCheckbox} 0 48u 100% 16u "同时永久删除本机 LightDo 的任务和设置"
 Pop $DeleteDataBox
 nsDialogs::Show
FunctionEnd
Function un.DataLeave
 ${NSD_GetState} $DeleteDataBox $DeleteData
FunctionEnd
Section "Uninstall"
 DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "LightDo"
 Delete "$SMPROGRAMS\LightDo\轻待 LightDo.lnk"
 Delete "$SMPROGRAMS\LightDo\卸载轻待.lnk"
 RMDir "$SMPROGRAMS\LightDo"
 Delete "$DESKTOP\轻待 LightDo.lnk"
 Delete "$INSTDIR\lightdo.exe"
 Delete "$INSTDIR\LICENSE"
 Delete "$INSTDIR\THIRD_PARTY_NOTICES.md"
 Delete "$INSTDIR\THIRD_PARTY_DEPENDENCIES.txt"
 Delete "$INSTDIR\licenses\LICENSE-Glance"
 Delete "$INSTDIR\licenses\LICENSE-QuietDo"
 Delete "$INSTDIR\licenses\*"
 RMDir "$INSTDIR\licenses"
 Delete "$INSTDIR\uninstall.exe"
 RMDir "$INSTDIR"
 DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LightDo"
 DeleteRegKey HKCU "Software\LightDo"
 ${If} $DeleteData == ${BST_CHECKED}
  RMDir /r "$APPDATA\LightDo"
 ${EndIf}
SectionEnd
