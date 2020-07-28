const NETWORKING_HOSTNAME: &str = "{{hostname}}
";

const NETWORKING_HOSTS: &str = "### Managed by compute

127.0.1.1 {{hostname}}
127.0.0.1 localhost

{% for host in hosts %}{{host.addr}} {% for name in host.names %}{{name}} {% endfor %}{% endfor %}

# The following lines are desirable for IPv6 capable hosts
::1 ip6-localhost ip6-loopback
fe00::0 ip6-localnet
ff00::0 ip6-mcastprefix
ff02::1 ip6-allnodes
ff02::2 ip6-allrouters
ff02::3 ip6-allhosts
";

pub enum Template {
    NetworkingHostname,
    NetworkingHosts,
}

impl Template {
    pub fn get_contents(&self) -> &str {
        match self {
            Self::NetworkingHostname => NETWORKING_HOSTNAME,
            Self::NetworkingHosts => NETWORKING_HOSTS,
        }
    }
}
