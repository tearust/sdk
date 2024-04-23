Tea-party 主要用到了 payment-channel的对应API

注意TEA系统的api基本都需要auth证明，所以需要提前登录，然后获取到auth信息，才能进行后续的api调用。
登录请求部分会单独说明

TEA是一个纯异步系统，所有的请求都需要附带一个uuid作为后续请求一致性的标识。

### open_payment_channel
创建channel，这是一个TXN
```
  address:  // 当前用户的登录地址
  tappIdB64: // 当前登录对应的tapp ID，这里需要和之前用户登录时候传入的id一致
  authB64: // 在登录请求中返回的用户授权信息
  payeeAddress: // 这是收款人地址
  gracePeriod: // 这是这个channel的锁定时间，过了才可以terminal 这个channel
  fundRemaining: // channel的初始fund
  expireTime: // 过期时间，timestamp
  channelId: // 这是channel的唯一标识，这里是在UI端随机生成的，所以不是在服务端生成的。
  tappKey: // tapp的关键字，字符串，用来区别，teaparty默认传入的都是teachat
  roomKey: // room id，也是随机生成的，规则和channelId一致，都是ETH地址。所以同一地址可以创建多个room的。
```

此请求在Rust端的参数定义如下
```
pub struct PayerOpenChannelRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
	pub tapp_key: String,
	pub room_key: String,
	pub payee_address: String,
	pub grace_period: Option<u64>,
	pub fund_remaining: String,
	pub expire_time: String,
}
```

### query_channel_list_with_account
根据传入的地址返回channel列表，这是QUERY

```
  address:  // 当前用户的登录地址
  tappIdB64: // 当前登录对应的tapp ID，这里需要和之前用户登录时候传入的id一致
  authB64: // 在登录请求中返回的用户授权信息
```
此query会返回一个json数据结构，里面包含了所在payer_list, payee_list, 已经最新的ts（TEA系统的最新时间戳）

此请求在Rust端的参数定义如下
```
pub struct QueryChannelListWithAccountRequest {
	pub uuid: String,
	pub address: String,
	pub tapp_id_b64: String,
	pub auth_b64: String,
}
```

### payer_early_terminate
payer提前终止channel，这是TXN

```
  address: 
  tappIdB64: 
  authB64: 
  channelId: //这是channel的唯一标识
```

此请求在Rust端的参数定义如下
```
pub struct PayerEarlyTerminateRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
}
```

### terminate
Terminate channel，这是TXN

```
  address: 
  tappIdB64: 
  authB64: 
  channelId: //这是channel的唯一标识
```

此请求在Rust端的参数定义如下
```
pub struct PayerTerminateRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub auth_b64: String,

	pub channel_id: String,
}
```

### payee_update_payment
payee更新channel的fund，这是TXN

```
  address: 
  tappIdB64: 
  channelId: 
  sig: // 之前收到的payer的签名，只有这个签名合法才能更新fund，也就是说payee才能收到token。
  closeChannel: // 是否关闭channel
  newFundRemaining: // 更新之后的fund，这个value需要和sig匹配才能通过。
```

此请求在Rust端的参数定义如下
```
pub struct PayeeUpdatePaymentRequest {
	pub uuid: String,
	pub tapp_id_b64: String,
	pub address: String,
	pub channel_id: String,
	pub sig: String,
	pub close_channel: bool,
	pub new_fund_remaining: String,
}
```
