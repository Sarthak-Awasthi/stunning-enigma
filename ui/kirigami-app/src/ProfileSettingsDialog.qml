import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Dialog {
    id: root
    title: "Profile Settings - Environment Variables"
    modal: true
    parent: Overlay.overlay
    anchors.centerIn: parent
    width: 600
    height: 500
    
    property int profileId: -1
    
    signal envVarAdded(string key, string value)
    signal envVarRemoved(string key)
    
    function syncEnvVars(sourceModel) {
        internalModel.clear()
        for (let i = 0; i < sourceModel.count; i++) {
            let item = sourceModel.get(i)
            internalModel.append({
                key: item.key,
                value: item.value
            })
        }
    }
    
    ListModel {
        id: internalModel
    }
    
    ColumnLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.largeSpacing
        
        Label {
            text: "Custom Environment Variables"
            font.bold: true
            Layout.fillWidth: true
        }
        
        Label {
            text: "These variables will be applied when launching the game or launcher for this profile."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            font.italic: true
        }
        
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Kirigami.Theme.alternateBackgroundColor
            border.color: Kirigami.Theme.focusColor
            border.width: 1
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 30
                    color: Kirigami.Theme.backgroundColor
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Kirigami.Units.smallSpacing
                        anchors.rightMargin: Kirigami.Units.smallSpacing
                        
                        Label {
                            text: "Key"
                            Layout.fillWidth: true
                            font.bold: true
                        }
                        Label {
                            text: "Value"
                            Layout.fillWidth: true
                            font.bold: true
                        }
                        Label {
                            text: ""
                            Layout.preferredWidth: 40
                        }
                    }
                }
                
                ListView {
                    id: envVarsListView
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    model: internalModel
                    clip: true
                    
                    delegate: Rectangle {
                        width: envVarsListView.width
                        height: 36
                        color: index % 2 === 0 ? "transparent" : Kirigami.Theme.alternateBackgroundColor
                        
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Kirigami.Units.smallSpacing
                            anchors.rightMargin: Kirigami.Units.smallSpacing
                            
                            TextField {
                                id: keyField
                                text: model.key
                                placeholderText: "e.g., PROTON_LOG"
                                Layout.fillWidth: true
                                readOnly: true
                                background: Rectangle {
                                    color: "transparent"
                                }
                            }
                            
                            TextField {
                                id: valueField
                                text: model.value
                                placeholderText: "e.g., 1"
                                Layout.fillWidth: true
                                readOnly: true
                                background: Rectangle {
                                    color: "transparent"
                                }
                            }
                            
                            Button {
                                icon.name: "list-remove"
                                Layout.preferredWidth: 40
                                onClicked: {
                                    root.envVarRemoved(model.key)
                                }
                            }
                        }
                    }
                }
            }
        }
        
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            
            TextField {
                id: newKeyField
                placeholderText: "Key (e.g., WINEDLLOVERRIDES)"
                Layout.fillWidth: true
            }
            
            TextField {
                id: newValueField
                placeholderText: "Value (e.g., d3d11=n,b)"
                Layout.fillWidth: true
            }
            
            Button {
                text: "Add"
                icon.name: "list-add"
                onClicked: {
                    if (newKeyField.text.trim() !== "" && newValueField.text.trim() !== "") {
                        root.envVarAdded(newKeyField.text.trim(), newValueField.text.trim())
                        newKeyField.clear()
                        newValueField.clear()
                    }
                }
            }
        }
        
        Dialog.buttonBox {
            standardButtons: Dialog.Close
        }
    }
}
