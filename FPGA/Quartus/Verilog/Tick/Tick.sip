set_global_assignment -entity "Tick" -library "lib_Tick" -name IP_TOOL_NAME "altera_pll"
set_global_assignment -entity "Tick" -library "lib_Tick" -name IP_TOOL_VERSION "23.1"
set_global_assignment -entity "Tick" -library "lib_Tick" -name IP_TOOL_ENV "mwpim"
set_global_assignment -library "lib_Tick" -name SPD_FILE [file join $::quartus(sip_path) "Tick.spd"]

set_global_assignment -library "lib_Tick" -name MISC_FILE [file join $::quartus(sip_path) "Tick_sim/Tick.vo"]
