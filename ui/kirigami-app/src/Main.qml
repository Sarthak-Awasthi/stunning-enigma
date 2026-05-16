import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtCore
import org.kde.kirigami as Kirigami
import ModManager 1.0

Kirigami.ApplicationWindow {
    id: root
    width: 1400
    height: 900
    visible: true
    title: "Mod Manager - Fallout 4"

    property string activeGameDataDir: ""
    property bool pendingLaunch: false
    property bool useF4seCache: true

    // --- State Models ---
    ListModel { id: profilesModel }
    ListModel { id: executablesModel }
    ListModel { id: modPriorityModel }
    ListModel { id: pluginsModel }

    // --- File Dialog for Mod Installation ---
    FileDialog {
        id: installModDialog
        title: "Select Mod Archive"
        nameFilters: ["Mod Archives (*.zip *.7z)"]
        onAccepted: {
            // Convert file:// URI to standard absolute Linux path
            let path = selectedFile.toString().replace(/^(file:\/{2})/, "")
            path = decodeURIComponent(path)
            appendLog(`Ingesting mod: ${path}...`)
            send("ingest_mod", { archive_path: path })
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // 1. Top Toolbar (Cleaned up and UX improved)
        ToolBar {
            Layout.fillWidth: true
            
            RowLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.largeSpacing
                spacing: Kirigami.Units.largeSpacing // Space between major groups

                // Group 1: Install Mod
                Button {
                    text: "Install Mod"
                    icon.name: "archive-insert"
                    Layout.alignment: Qt.AlignVCenter
                    onClicked: installModDialog.open()
                }

                ToolSeparator {
                    Layout.alignment: Qt.AlignVCenter
                }

                // Group 2: Profile Selection
                RowLayout {
                    spacing: Kirigami.Units.smallSpacing
                    Layout.alignment: Qt.AlignVCenter

                    Label { 
                        text: "Profile:" 
                        Layout.alignment: Qt.AlignVCenter
                    }
                    ComboBox {
                        id: profileComboBox
                        Layout.preferredWidth: 250
                        Layout.alignment: Qt.AlignVCenter
                        model: profilesModel
                        textRole: "name"
                        onActivated: root.loadProfileData(currentValue)
                    }
                    Button {
                        icon.name: "configure"
                        text: "Configure"
                        ToolTip.text: "Profile Settings"
                        Layout.alignment: Qt.AlignVCenter
                    }
                }

                Item { Layout.fillWidth: true } // Expands to push the Launch group to the right

                // Group 3: Launch Section
                RowLayout {
                    spacing: Kirigami.Units.smallSpacing
                    Layout.alignment: Qt.AlignVCenter

                    ComboBox {
                        id: executableComboBox
                        Layout.preferredWidth: 220
                        Layout.alignment: Qt.AlignVCenter
                        model: ["Fallout 4", "F4SE", "Fallout 4 Launcher"]
                    }
                    Button {
                        text: "Run"
                        icon.name: "media-playback-start"
                        highlighted: true
                        Layout.preferredWidth: 120
                        Layout.alignment: Qt.AlignVCenter
                        onClicked: {
                            const profileId = root.currentProfileId();
                            if (profileId < 0) return;
                            
                            useF4seCache = (executableComboBox.currentText === "F4SE");
                            
                            if (activeGameDataDir === "") {
                                appendLog("Error: Game not detected. Cannot deploy.");
                                return;
                            }

                            appendLog("Preparing Virtual File System...");
                            pendingLaunch = true;
                            
                            send("deploy_apply", {
                                profile_id: profileId,
                                game_data_dir: activeGameDataDir
                            });
                        }
                    }
                }
            }
        }

        // 2. Main Dual-Pane Area wrapped in DropArea for Drag-and-Drop
        DropArea {
            Layout.fillWidth: true
            Layout.fillHeight: true
            keys: ["text/uri-list"]

            onDropped: (drop) => {
                if (drop.hasUrls) {
                    for (let i = 0; i < drop.urls.length; i++) {
                        let path = drop.urls[i].toString().replace(/^(file:\/{2})/, "")
                        path = decodeURIComponent(path)
                        
                        if (path.endsWith(".zip") || path.endsWith(".7z")) {
                            appendLog(`Dropped archive: ${path}`)
                            send("ingest_mod", { archive_path: path })
                        } else {
                            appendLog(`Ignored dropped file (unsupported format): ${path}`)
                        }
                    }
                    drop.accept()
                }
            }

            SplitView {
                anchors.fill: parent
                orientation: Qt.Horizontal

                // LEFT PANE: Mod Priority List
                Frame {
                    SplitView.fillWidth: true
                    SplitView.preferredWidth: root.width * 0.6
                    padding: 0

                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 0
                        
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 30
                            color: Kirigami.Theme.alternateBackgroundColor
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Kirigami.Units.smallSpacing
                                anchors.rightMargin: Kirigami.Units.smallSpacing
                                Label { text: "Mod Name"; Layout.fillWidth: true }
                                Label { text: "Conflicts"; Layout.preferredWidth: 80 }
                                Label { text: "Category"; Layout.preferredWidth: 120 }
                                Label { text: "Priority"; Layout.preferredWidth: 60 }
                            }
                        }

                        ListView {
                            id: modListView
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            model: modPriorityModel
                            clip: true
                            boundsBehavior: Flickable.StopAtBounds

                            delegate: Rectangle {
                                width: modListView.width
                                height: 32
                                color: index % 2 === 0 ? "transparent" : Kirigami.Theme.alternateBackgroundColor

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: Kirigami.Units.smallSpacing
                                    anchors.rightMargin: Kirigami.Units.smallSpacing

                                    CheckBox {
                                        checked: model.enabled
                                        Layout.alignment: Qt.AlignVCenter
                                        onToggled: send("set_profile_mod_enabled", {
                                            profile_id: root.currentProfileId(),
                                            mod_id: model.modId,
                                            enabled: checked
                                        })
                                    }
                                    Label { 
                                        text: model.modName
                                        Layout.fillWidth: true
                                        Layout.alignment: Qt.AlignVCenter
                                        font.bold: model.isSeparator !== undefined ? model.isSeparator : false
                                    }
                                    Label { 
                                        text: model.conflicts ? "⚡" : ""
                                        Layout.preferredWidth: 80
                                        Layout.alignment: Qt.AlignVCenter
                                    }
                                    Label { 
                                        text: model.category
                                        Layout.preferredWidth: 120
                                        Layout.alignment: Qt.AlignVCenter
                                    }
                                    Label { 
                                        text: model.priority
                                        Layout.preferredWidth: 60
                                        Layout.alignment: Qt.AlignVCenter
                                    }
                                }
                            }
                        }
                    }
                }

                // RIGHT PANE: Plugins / Data
                Frame {
                    SplitView.preferredWidth: root.width * 0.4
                    padding: 0

                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 0

                        TabBar {
                            id: rightTabBar
                            Layout.fillWidth: true
                            TabButton { text: "Plugins" }
                            TabButton { text: "Data (VFS)" }
                            TabButton { text: "Downloads" }
                        }

                        StackLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            currentIndex: rightTabBar.currentIndex

                            // Tab 1: Plugins
                            Item {
                                ColumnLayout {
                                    anchors.fill: parent
                                    spacing: 0
                                    
                                    Button {
                                        text: "Sort with LOOT"
                                        icon.name: "view-sort-ascending"
                                        Layout.fillWidth: true
                                        Layout.margins: Kirigami.Units.smallSpacing
                                        onClicked: send("sort_with_loot", { profile_id: currentProfileId() })
                                    }
                                    ListView {
                                        id: pluginListView
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        model: pluginsModel
                                        clip: true
                                        boundsBehavior: Flickable.StopAtBounds
                                        
                                        delegate: Item {
                                            width: pluginListView.width
                                            height: 32
                                            
                                            Rectangle {
                                                anchors.fill: parent
                                                color: index % 2 === 0 ? "transparent" : Kirigami.Theme.alternateBackgroundColor
                                            }
                                            
                                            CheckBox {
                                                anchors.verticalCenter: parent.verticalCenter
                                                anchors.left: parent.left
                                                anchors.leftMargin: Kirigami.Units.smallSpacing
                                                text: `[${model.loadIndex}] ${model.filename} (${model.kind.toUpperCase()})`
                                                checked: model.enabled
                                                onToggled: {
                                                    send("set_profile_plugin_enabled", {
                                                        profile_id: root.currentProfileId(),
                                                        plugin_id: model.pluginId,
                                                        enabled: checked
                                                    })
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Tab 2: Data (Virtual File System)
                            Item {
                                Label {
                                    anchors.centerIn: parent
                                    text: "File tree view goes here\n(Requires backend VFS API)"
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Bottom Area: Logs & Conflict Details
        SplitView {
            Layout.fillWidth: true
            Layout.preferredHeight: 150
            orientation: Qt.Horizontal

            TextArea {
                id: logArea
                SplitView.fillWidth: true
                SplitView.preferredWidth: root.width * 0.6
                readOnly: true
                font.family: "monospace"
                text: "Mod Manager Initialized...\n"
                
                background: Rectangle {
                    color: Kirigami.Theme.backgroundColor
                    border.color: Kirigami.Theme.focusColor
                    border.width: 1
                }
            }

            TextArea {
                id: conflictDetailArea
                SplitView.fillWidth: true
                SplitView.preferredWidth: root.width * 0.4
                readOnly: true
                placeholderText: "Select a mod to view conflict details..."
                
                background: Rectangle {
                    color: Kirigami.Theme.backgroundColor
                    border.color: Kirigami.Theme.focusColor
                    border.width: 1
                }
            }
        }
    }

    IpcClient {
        id: ipcClient
        onResponseReceived: (response) => root.handleIpcResponse(response)
    }

    function send(method, params) {
        ipcClient.call(method, params)
    }

    function appendLog(msg) {
        logArea.text += `[${new Date().toLocaleTimeString()}] ${msg}\n`
        logArea.cursorPosition = logArea.length // Auto-scroll to bottom
    }
    
    Component.onCompleted: {
        send("detect_game", {}) 
        send("list_profiles", { instance_id: 1 }) 
    }

    function handleIpcResponse(response) {
        if (response.type === "profile_list") {
            profilesModel.clear()
            response.payload.profiles.forEach(p => profilesModel.append(p))
            if(profilesModel.count > 0) {
                profileComboBox.currentIndex = 0
                loadProfileData(profilesModel.get(0).id)
            }
        }
        else if (response.type === "profile_mods") {
            modPriorityModel.clear()
            response.payload.mods.forEach(m => modPriorityModel.append({
                modId: m.mod_id,
                modName: m.mod_name,
                enabled: m.enabled,
                priority: m.priority,
                category: "Unassigned",
                conflicts: false
            }))
        }
        else if (response.type === "game_detected") {
            activeGameDataDir = response.payload.install_path + "/Data";
            appendLog(`Linked game at: ${response.payload.install_path}`);
        }
        else if (response.type === "profile_plugins") {
            pluginsModel.clear()
            response.payload.plugins.forEach(p => pluginsModel.append({
                pluginId: p.plugin_id,
                filename: p.filename,
                kind: p.kind,
                enabled: p.enabled,
                loadIndex: p.load_index
            }))
        }
        else if (response.type === "profile_plugin_updated" || response.type === "profile_mod_updated") {
            loadProfileData(root.currentProfileId())
        }
        else if (response.type === "mod_ingested") {
            appendLog(`Success: Installed '${response.payload.name}' (${response.payload.file_count} files)`)
            send("upsert_profile_mod", {
                profile_id: root.currentProfileId(),
                mod_id: response.payload.mod_id,
                enabled: true, 
                priority: modPriorityModel.count 
            })
        }
        else if (response.type === "plugins_sorted") {
            appendLog(`Plugins sorted successfully via ${response.payload.engine}.`)
            send("write_load_order", { profile_id: root.currentProfileId() })
            send("list_profile_plugins", { profile_id: root.currentProfileId() })
        }
        else if (response.type === "deploy_applied") {
            appendLog(`Virtual File System Deployed (Manifest ${response.payload.manifest_id})`);
            if (pendingLaunch) {
                pendingLaunch = false;
                appendLog("Starting game executable...");
                send("launch_game", { 
                    profile_id: root.currentProfileId(), 
                    use_f4se: useF4seCache 
                });
            }
        }
        else if (response.type === "error") {
            appendLog(`ERROR: ${response.payload.message}`)
        }
    }

    function currentProfileId() {
        if(profileComboBox.currentIndex >= 0)
            return profilesModel.get(profileComboBox.currentIndex).id
        return -1
    }

    function loadProfileData(profileId) {
        send("list_profile_mods", { profile_id: profileId })
        send("list_profile_plugins", { profile_id: profileId })
    }
}