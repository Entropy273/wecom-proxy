# wecom-proxy

Rust version of [wecomchan](https://github.com/easychen/wecomchan). 

Just call the API, and the message will be sent to your WeChat & WeCom.

## Usage

```bash
docker run -d -p {}:3000 --name wecom-proxy \
-e AUTH_KEY="{}" \
-e WECOM_CID="{}" \
-e WECOM_AID="{}" \
-e WECOM_SECRET="{}" \
entropy273/wecom-proxy
```

- `AUTH_KEY`: auth key. Set by yourself to protect the proxy.
- `WECOM_CID`: corpid of WeCom.
- `WECOM_AID`: appid of WeCom Application.
- `WECOM_SECRET`: secret of WeCom Application.

For details on obtaining these parameters, refer to [wecomchan](https://github.com/easychen/wecomchan).

## API

### `GET` **/wecom**

- `auth_key`: auth key
- `msg`: message to send

```bash
curl -X GET http://localhost:3000/wecom?auth_key=auth_key&msg=message
```

### `POST` **/wecom**

Content-Type: application/json

- `auth_key`: auth key
- `msg`: message to send

```bash
curl -X POST -H "Content-Type: application/json" -d '{"auth_key": "auth_key", "msg": "message"}' http://localhost:3000/wecom
```
