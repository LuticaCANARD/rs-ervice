
/// 하나의 서비스 컨테이너... 
/// 서비스는 하나의 impl을 가지며, 다양한 함수들을 보유한다.
pub struct Service<TActor>
where TActor: 'static + Send + Sync,
{
    pub name: String,
    pub actor: TActor,
}

/// 하나의 서비스에서 관리할 이벤트들의 집합
pub struct ServiceCluster<TActor>
where TActor: 'static + Send + Sync,
{
    pub services: Vec<Service<TActor>>,
    pub name: String,
}