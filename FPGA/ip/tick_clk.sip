set_global_assignment -entity "tick_clk" -library "lib_tick_clk" -name IP_TOOL_NAME "altera_pll"
set_global_assignment -entity "tick_clk" -library "lib_tick_clk" -name IP_TOOL_VERSION "23.1"
set_global_assignment -entity "tick_clk" -library "lib_tick_clk" -name IP_TOOL_ENV "mwpim"
set_global_assignment -library "lib_tick_clk" -name SPD_FILE [file join $::quartus(sip_path) "tick_clk.spd"]

set_global_assignment -library "lib_tick_clk" -name MISC_FILE [file join $::quartus(sip_path) "tick_clk_sim/tick_clk.vo"]
