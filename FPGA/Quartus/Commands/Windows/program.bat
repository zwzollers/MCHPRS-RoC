cd /d "%~dp0"
cd ..
cd ..
set PATH=%PATH%;C:\intelFPGA_lite\23.1std\quartus\bin64
quartus_pgm -c "DE-SoC [USB-1]" -m jtag -o "p;FPGA-MCHPRS.sof@2"