#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <qqml.h>

#include "ipc_client.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    QCoreApplication::setOrganizationName(QStringLiteral("ModManager"));
    QCoreApplication::setOrganizationDomain(QStringLiteral("modmanager.local"));
    QCoreApplication::setApplicationName(QStringLiteral("ModManager"));

    QQmlApplicationEngine engine;
    qmlRegisterType<IpcClient>("ModManager", 1, 0, "IpcClient");
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection
    );

    engine.loadFromModule(QStringLiteral("ModManager"), QStringLiteral("Main"));
    return app.exec();
}
