use super::*;

pub(super) fn handle_with_access_gate<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
    handler: impl FnOnce(ApiRequest<'request, 'connection>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let path = request_path_without_query(request.connection().uri()).to_owned();
    let input = access_input(&mut request);
    match plan_http_access(input) {
        HttpAccessDecision::Allow => handler(request),
        HttpAccessDecision::Deny(response) => {
            log_access_denied("http", &path, input);
            send_public_response(request, response)
        }
    }
}

pub(super) fn access_input<'request, 'connection>(
    request: &mut ApiRequest<'request, 'connection>,
) -> RouteAccessInput {
    let raw_request = (*request.connection()).handle();
    access_input_from_raw(raw_request)
}

pub(super) fn access_input_from_raw(request: *mut sys::httpd_req_t) -> RouteAccessInput {
    let request_ip = maybe_peer_ipv4(request).unwrap_or(Ipv4Addr::UNSPECIFIED);

    RouteAccessInput {
        ap_mode_enabled: ap_mode_enabled(),
        request_ip,
        origin: origin_gate_from_raw(request),
    }
}

pub(super) fn ap_mode_enabled() -> bool {
    let mut mode = 0;
    let result = unsafe { sys::esp_wifi_get_mode(&mut mode) };
    result == sys::ESP_OK
        && matches!(
            mode,
            sys::wifi_mode_t_WIFI_MODE_AP | sys::wifi_mode_t_WIFI_MODE_APSTA
        )
}

pub(super) fn maybe_peer_ipv4(request: *mut sys::httpd_req_t) -> Option<Ipv4Addr> {
    unsafe {
        let sockfd = sys::httpd_req_to_sockfd(request);
        if sockfd == -1 {
            log::warn!("axeos_access_gate_peer_ip=unavailable reason=no_socket");
            return None;
        }

        let mut addr = sys::sockaddr_in {
            sin_len: core::mem::size_of::<sys::sockaddr_in>() as _,
            sin_family: sys::AF_INET as _,
            ..Default::default()
        };
        let mut addr_len = core::mem::size_of::<sys::sockaddr_in>() as sys::socklen_t;

        if sys::lwip_getpeername(
            sockfd,
            &mut addr as *mut _ as *mut sys::sockaddr,
            &mut addr_len,
        ) != sys::ESP_OK
        {
            log::warn!("axeos_access_gate_peer_ip=unavailable reason=getpeername_failed");
            return None;
        }

        let raw_addr = addr.sin_addr.s_addr;
        let normalization = normalize_peer_ipv4(raw_addr);
        if let PeerIpv4Normalization::HostOrderFallback {
            address,
            network_order_address,
        } = normalization
        {
            log::warn!(
                "axeos_access_gate_peer_ip_byte_order=host_order raw=0x{raw_addr:08x} network_order_ip={network_order_address} host_order_ip={address}"
            );
        }

        Some(normalization.address())
    }
}

pub(super) fn origin_gate_from_raw(request: *mut sys::httpd_req_t) -> OriginGate {
    let mut buffer = [0; ORIGIN_HEADER_BUFFER_BYTES];
    let result = unsafe {
        sys::httpd_req_get_hdr_value_str(
            request,
            ORIGIN_HEADER.as_ptr().cast(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };
    match result {
        sys::ESP_OK => {}
        sys::ESP_ERR_NOT_FOUND => return OriginGate::Missing,
        sys::ESP_ERR_HTTPD_RESULT_TRUNC => return OriginGate::Invalid,
        _ => return OriginGate::Invalid,
    }

    let Ok(origin) = (unsafe { CStr::from_ptr(buffer.as_ptr()) }).to_str() else {
        return OriginGate::Invalid;
    };

    origin_gate_from_header(origin)
}
