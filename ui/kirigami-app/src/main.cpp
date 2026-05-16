#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <qqml.h>

#include "ipc_client.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;
    qmlRegisterType<IpcClient>("ModManager", 1, 0, "IpcClient");

    const QUrl url(u"qrc:/qt/qml/ModManager/src/Main.qml"_qs);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection
    );

    engine.load(url);
    return app.exec();
}
