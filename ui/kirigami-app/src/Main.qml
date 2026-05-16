import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import ModManager 1.0

Kirigami.ApplicationWindow {
    id: root
    width: 1100
    height: 760
    visible: true
    title: "Mod Manager"

    IpcClient {
        id: ipcClient
        onResponseReceived: (response) => {
            outputArea.text = JSON.stringify(response, null, 2)
        }
        onRequestFailed: (message) => {
            outputArea.text = "IPC error: " + message
        }
    }

    function send(method, params) {
        outputArea.text = "Sending: " + method + "\nPayload:\n" + JSON.stringify(params, null, 2) + "\n\n"
        ipcClient.call(method, params || {})
    }

    readonly property var methodTemplates: [
        { "name": "ping", "params": {} },
        { "name": "detect_game", "params": {} },
        { "name": "register_game", "params": { "path": "/path/to/Fallout 4" } },
        { "name": "list_runners", "params": {} },
        { "name": "pin_runner", "params": { "profile_id": 1, "runner_id": 1 } },
        { "name": "create_profile", "params": { "instance_id": 1, "name": "Default" } },
        { "name": "list_profiles", "params": { "instance_id": 1 } },
        { "name": "delete_profile", "params": { "profile_id": 1 } },
        { "name": "ingest_mod", "params": { "archive_path": "/path/to/mod.zip" } },
        { "name": "list_profile_mods", "params": { "profile_id": 1 } },
        { "name": "upsert_profile_mod", "params": { "profile_id": 1, "mod_id": 1, "enabled": true, "priority": 100 } },
        { "name": "set_profile_mod_enabled", "params": { "profile_id": 1, "mod_id": 1, "enabled": true } },
        { "name": "set_profile_mod_priority", "params": { "profile_id": 1, "mod_id": 1, "priority": 100 } },
        { "name": "deploy_preview", "params": { "profile_id": 1, "game_data_dir": "/path/to/Fallout 4/Data" } },
        { "name": "deploy_apply", "params": { "profile_id": 1, "game_data_dir": "/path/to/Fallout 4/Data" } },
        { "name": "deploy_rollback", "params": { "manifest_id": 1 } },
        { "name": "sync_plugins", "params": { "profile_id": 1, "data_dir": "/path/to/Fallout 4/Data" } },
        { "name": "validate_plugins", "params": { "profile_id": 1 } },
        { "name": "sort_with_loot", "params": { "profile_id": 1 } },
        { "name": "write_load_order", "params": { "profile_id": 1 } },
        { "name": "launch_preflight", "params": { "profile_id": 1, "use_f4se": true } },
        { "name": "launch_game", "params": { "profile_id": 1, "use_f4se": true } }
    ]

    function selectedMethod() {
        if (methodPicker.currentIndex < 0 || methodPicker.currentIndex >= methodTemplates.length) {
            return null
        }
        return methodTemplates[methodPicker.currentIndex]
    }

    pageStack.initialPage: Kirigami.ScrollablePage {
        title: "Dashboard"

        ColumnLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.largeSpacing

            GridLayout {
                columns: 3
                columnSpacing: Kirigami.Units.largeSpacing
                rowSpacing: Kirigami.Units.smallSpacing
                Layout.fillWidth: true

                Label {
                    text: "Method"
                    Layout.alignment: Qt.AlignVCenter
                }
                ComboBox {
                    id: methodPicker
                    Layout.columnSpan: 2
                    Layout.fillWidth: true
                    model: root.methodTemplates
                    textRole: "name"
                    onCurrentIndexChanged: {
                        const selected = root.selectedMethod()
                        payloadArea.text = selected ? JSON.stringify(selected.params, null, 2) : "{}"
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Kirigami.Units.largeSpacing

                Button {
                    text: "Send Request"
                    onClicked: {
                        const selected = root.selectedMethod()
                        if (!selected) {
                            outputArea.text = "No method selected"
                            return
                        }

                        let payload = {}
                        const raw = payloadArea.text.trim()
                        if (raw.length > 0) {
                            try {
                                payload = JSON.parse(raw)
                            } catch (e) {
                                outputArea.text = "Invalid JSON payload: " + e
                                return
                            }
                        }

                        root.send(selected.name, payload)
                    }
                }

                Button {
                    text: "Reset Payload"
                    onClicked: {
                        const selected = root.selectedMethod()
                        payloadArea.text = selected ? JSON.stringify(selected.params, null, 2) : "{}"
                    }
                }

                Label {
                    text: "Socket: " + ipcClient.socketPath
                    Layout.fillWidth: true
                    elide: Text.ElideMiddle
                }
            }

            TextArea {
                id: payloadArea
                Layout.fillWidth: true
                Layout.preferredHeight: 220
                wrapMode: TextEdit.NoWrap
                placeholderText: "JSON payload for selected method"
            }

            TextArea {
                id: outputArea
                Layout.fillWidth: true
                Layout.fillHeight: true
                readOnly: true
                wrapMode: TextEdit.NoWrap
                placeholderText: "Daemon responses will appear here."
            }
        }
    }

    Component.onCompleted: {
        methodPicker.currentIndex = 0
    }
}
