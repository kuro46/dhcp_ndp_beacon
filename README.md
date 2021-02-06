# dhcp_ndp_beacon

REST API server of DHCP leases and NDP neighbors for FreeBSD

Reads `/var/db/dhcpd/dhcpd.leases` for retrieve DHCP leases  
Executes `ndp -a` for retrieve NDP neighbors

## API spec

```sh
$ curl 192.168.0.1/api/status
{
  "aa:bb:cc:dd:ee:ff": {
    "ndp_entries": [ # maybe empty
      {
        "mac_address": "aa:bb:cc:dd:ee:ff",
        "ipv6_address": "global address"
      },
      {
        "mac_address": "aa:bb:cc:dd:ee:ff",
        "ipv6_address": "link-local address%interface"
      }
    ],
    "dhcp_lease": { # nullable
      "mac_address": "aa:bb:cc:dd:ee:ff",
      "ipv4_address": "192.168.0.29", # address
      "end": "2021/02/06 11:17:27", # lease end
      "host_name": null # hostname string, nullable
    }
  },
  "mac address 2": {
    ...
  }
}
```
