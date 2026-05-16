#pragma once

#include <QObject>
#include <QVariant>

class IpcClient : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString socketPath READ socketPath WRITE setSocketPath NOTIFY socketPathChanged)

public:
    explicit IpcClient(QObject *parent = nullptr);

    QString socketPath() const;
    void setSocketPath(const QString &value);

    Q_INVOKABLE void call(const QString &method, const QVariantMap &params = QVariantMap());

signals:
    void socketPathChanged();
    void responseReceived(const QVariant &response);
    void requestFailed(const QString &message);

private:
    QString m_socketPath;
};
