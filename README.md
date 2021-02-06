# dhcp_ndp_beacon

REST API server of DHCP leases and NDP neighbors for FreeBSD

Reads `/var/db/dhcpd/dhcpd.leases` for retrieve DHCP leases  
Executes `ndp -a` for retrieve NDP neighbors
