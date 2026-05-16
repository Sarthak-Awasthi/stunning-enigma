#include "ipc_client.h"

#include <QElapsedTimer>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalSocket>

IpcClient::IpcClient(QObject *parent)
    : QObject(parent),
      m_socketPath("/tmp/mm-daemon.sock") {}

QString IpcClient::socketPath() const {
    return m_socketPath;
}

void IpcClient::setSocketPath(const QString &value) {
    if (m_socketPath == value) {
        return;
    }

    m_socketPath = value;
    emit socketPathChanged();
}

void IpcClient::call(const QString &method, const QVariantMap &params) {
    if (method.trimmed().isEmpty()) {
        emit requestFailed("Method must not be empty");
        return;
    }

    QJsonObject request;
    request.insert("method", method);
    if (!params.isEmpty()) {
        request.insert("params", QJsonObject::fromVariantMap(params));
    }

    QLocalSocket socket;
    socket.connectToServer(m_socketPath);
    if (!socket.waitForConnected(3000)) {
        emit requestFailed(QString("Failed to connect to %1: %2").arg(m_socketPath, socket.errorString()));
        return;
    }

    QByteArray encoded = QJsonDocument(request).toJson(QJsonDocument::Compact);
    encoded.append('\n');

    if (socket.write(encoded) == -1 || !socket.waitForBytesWritten(3000)) {
        emit requestFailed(QString("Failed to send request: %1").arg(socket.errorString()));
        return;
    }

    QByteArray buffer;
    QElapsedTimer timer;
    timer.start();

    while (timer.elapsed() < 6000) {
        if (!socket.waitForReadyRead(250)) {
            continue;
        }

        buffer.append(socket.readAll());
        const int newline = buffer.indexOf('\n');
        if (newline == -1) {
            continue;
        }

        const QByteArray line = buffer.left(newline).trimmed();
        if (line.isEmpty()) {
            emit requestFailed("Received empty response line from daemon");
            return;
        }

        QJsonParseError parseError;
        const QJsonDocument doc = QJsonDocument::fromJson(line, &parseError);
        if (parseError.error != QJsonParseError::NoError) {
            emit requestFailed(QString("Failed to parse daemon response: %1").arg(parseError.errorString()));
            return;
        }

        emit responseReceived(doc.toVariant());
        return;
    }

    emit requestFailed("Timed out waiting for daemon response");
}
