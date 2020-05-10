extends Node

var gdn

func _ready():
    gdn = GDNative.new()
    var status = false

    gdn.library = load("res://libdeso3d.gdnlib")

    if gdn.initialize():
        status = gdn.call_native("standard_varcall", "run_tests", [])
        gdn.terminate()

    if status:
        print('all tests passed')
    else:
        OS.exit_code = 1
        print('test failure')

    get_tree().quit()
