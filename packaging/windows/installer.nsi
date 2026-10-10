; GoharScribe Windows Installer
; Built with NSIS 3.x

!include "MUI2.nsh"

Name "GoharScribe"
OutFile "GoharScribe-Setup.exe"
InstallDir "$PROGRAMFILES\GoharScribe"
RequestExecutionLevel admin

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "GoharScribe" SecMain
    SetOutPath "$INSTDIR"
    File "goharscribe.exe"
    File "goharscribe-cli.exe"
    
    ; Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\GoharScribe"
    CreateShortcut "$SMPROGRAMS\GoharScribe\GoharScribe.lnk" "$INSTDIR\goharscribe.exe"
    CreateShortcut "$SMPROGRAMS\GoharScribe\Uninstall.lnk" "$INSTDIR\uninstall.exe"
    
    ; Desktop shortcut
    CreateShortcut "$DESKTOP\GoharScribe.lnk" "$INSTDIR\goharscribe.exe"
    
    ; File associations
    WriteRegStr HKCR ".gsdoc" "" "GoharScribe.Document"
    WriteRegStr HKCR "GoharScribe.Document" "" "GoharScribe Document"
    WriteRegStr HKCR "GoharScribe.Document\shell\open\command" "" '"$INSTDIR\goharscribe.exe" "%1"'
    
    ; Uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoharScribe" "DisplayName" "GoharScribe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoharScribe" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoharScribe" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoharScribe" "NoRepair" 1
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\goharscribe.exe"
    Delete "$INSTDIR\goharscribe-cli.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    
    Delete "$SMPROGRAMS\GoharScribe\GoharScribe.lnk"
    Delete "$SMPROGRAMS\GoharScribe\Uninstall.lnk"
    RMDir "$SMPROGRAMS\GoharScribe"
    Delete "$DESKTOP\GoharScribe.lnk"
    
    DeleteRegKey HKCR ".gsdoc"
    DeleteRegKey HKCR "GoharScribe.Document"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoharScribe"
SectionEnd
