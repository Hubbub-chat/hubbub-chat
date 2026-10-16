//! SSRF Protection and URL validation guard.

use std::net::IpAddr;
use url::Url;

use crate::errors::PolicyError;

pub struct UrlGuard;

impl UrlGuard {
    /// Validate URL format, enforce HTTP/HTTPS, and check for obvious SSRF targets.
    pub fn validate_url(raw_url: &str) -> Result<Url, PolicyError> {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            return Err(PolicyError::InvalidUrl("URL cannot be empty".to_string()));
        }

        let parsed =
            Url::parse(trimmed).map_err(|e| PolicyError::InvalidUrl(format!("{e}: {trimmed}")))?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(PolicyError::DisallowedScheme(scheme.to_string()));
        }

        match parsed.host() {
            Some(url::Host::Ipv4(v4)) => {
                Self::check_ip_allowed(&IpAddr::V4(v4), trimmed)?;
            }
            Some(url::Host::Ipv6(v6)) => {
                Self::check_ip_allowed(&IpAddr::V6(v6), trimmed)?;
            }
            Some(url::Host::Domain(host)) => {
                // "localhost." is the fully qualified form of "localhost"
                let lower_host = host.trim_end_matches('.').to_lowercase();
                if lower_host == "localhost"
                    || lower_host.ends_with(".localhost")
                    || lower_host.ends_with(".local")
                    || lower_host.ends_with(".internal")
                    || lower_host == "metadata.google.internal"
                    || lower_host == "instance-data"
                {
                    return Err(PolicyError::SsrfBlocked {
                        url: trimmed.to_string(),
                        ip: host.to_string(),
                    });
                }
            }
            None => {
                return Err(PolicyError::InvalidUrl(format!(
                    "Missing host in URL: {trimmed}"
                )));
            }
        }

        Ok(parsed)
    }

    /// Check if an IP address belongs to private/internal/loopback/multicast ranges.
    pub fn is_private_or_restricted_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                v4.is_loopback()
                    || v4.is_private()
                    || v4.is_link_local()
                    || v4.is_multicast()
                    || v4.is_broadcast()
                    || v4.is_unspecified()
                    // 0.0.0.0/8 (Current network)
                    || octets[0] == 0
                    // 100.64.0.0/10 (Shared address space / Carrier-grade NAT)
                    || (octets[0] == 100 && (octets[1] & 0b1100_0000) == 64)
                    // 192.0.0.0/24 (IETF Protocol Assignments)
                    || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                    // 198.18.0.0/15 (Benchmarking)
                    || (octets[0] == 198 && (octets[1] & 0b1111_1110) == 18)
            }
            IpAddr::V6(v6) => {
                let segments = v6.segments();
                v6.is_loopback()
                    || v6.is_unspecified()
                    || v6.is_multicast()
                    // Link-local unicast fe80::/10
                    || (segments[0] & 0xffc0) == 0xfe80
                    // Unique local fc00::/7
                    || (segments[0] & 0xfe00) == 0xfc00
                    // IPv4-mapped IPv6 ::ffff:x.x.x.x
                    || match v6.to_ipv4_mapped() {
                        Some(mapped_v4) => Self::is_private_or_restricted_ip(&IpAddr::V4(mapped_v4)),
                        None => false,
                    }
            }
        }
    }

    /// Verify an IP is not private or restricted, returning a PolicyError if blocked.
    pub fn check_ip_allowed(ip: &IpAddr, url: &str) -> Result<(), PolicyError> {
        if Self::is_private_or_restricted_ip(ip) {
            return Err(PolicyError::SsrfBlocked {
                url: url.to_string(),
                ip: ip.to_string(),
            });
        }
        Ok(())
    }
}
