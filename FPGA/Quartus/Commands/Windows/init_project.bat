cd /d "%~dp0"
cd ..
cd ..
set PATH=%PATH%;C:\intelFPGA_lite\23.1std\quartus\bin64
quartus_sh -t create_project.tcl