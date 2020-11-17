use nrf52810_hal::pac;

pub struct Radio {
    radio: pac::RADIO,
    packet: [u8; 258]
}

impl Radio {
    pub fn new(radio: pac::RADIO) -> Radio {
        Radio {radio: radio, packet: [0; 258]}
    }
    
    pub fn power_off(&self) {
        self.radio.power.write(|w| { w.power().disabled() });
    }

    pub fn power_enabled(&self) -> bool {
        self.radio.power.read().power().bit()
    }

    pub fn init_transmission(&self) {
        // POWER
        // 1 (default)
        self.radio.power.write(|w| { w.power().enabled() });

        // MODE
        // MODE: data rate and modulation
        self.radio.mode.write(|w| { w.mode().ble_1mbit() });

        // FREQUENCY
        // FREQUENCY: [0..100] freq = 2400 MHz + freq => 90
        self.radio.frequency.write(|w| unsafe { w.frequency().bits(90) });

        // PCNF0
        // LFLEN: length field length in bits => 8
        // S0LEN: S0 length in bytes => 0 (default)
        // S1LEN: S1 length in bits => 0 (default)
        // S1INCL: 0 (default)
        // PLEN: 0 (default)
        self.radio.pcnf0.write(|w| unsafe {
            w.lflen().bits(8)
             .plen().bit(true)
        });

        // PCNF1
        // MAXLEN: max length of payload packet => 255
        // STATLEN: 0 (default)
        // BALEN: base address length => 4
        // ENDIAN: 0 (default) => 1
        // WHITEEN: 0 (default)
        self.radio.pcnf1.write(|w| unsafe {
            w.maxlen().bits(255)
             .balen().bits(4)
             .endian().bit(true)
        });

        // BASE0
        // BASE0: 0xABCDABCD
        self.radio.base0.write(|w| unsafe { w.base0().bits(0xABCDABCD) });

        // PREFIX0
        // AP0: 0xDA
        self.radio.prefix0.write(|w| unsafe { w.ap0().bits(0xEF) });

        // TXADDRESS
        // TXADDRESS: 0 (default)
        self.radio.txaddress.write(|w| unsafe { w.txaddress().bits(0) });

        // TXPOWER
        // TXPOWER: +4 dB
        self.radio.txpower.write(|w| { w.txpower().pos4d_bm() });

        // CRCCNF
        // LEN: length => 3
        // SKIPADDR: 0 (default)
        self.radio.crccnf.write(|w| w.len().bits(3));

        // CRCPOLY
        // x24 + x10 + x9 + x6 + x4 + x3 + x + 1
        // CRCPOLY: 00000000_00000110_01011011
        self.radio.crcpoly.write(|w| unsafe { w.bits(0b00000000_00000110_01011011) });

        // Shortcuts
        // READY - START
        // ADDRESS - RSSISTART
        self.radio.shorts.write(|w| {
            w.ready_start().bit(true)
             .end_disable().bit(true)
        });
    }

    pub fn start_transmission(&mut self, data: &[&[u8]]) {
        // copy data into buffer
        let mut len = 0;

        for part in data {
            for byte in *part {
                self.packet[len + 1] = *byte;
                len += 1;
            }
        }

        self.packet[0] = (len + 1) as u8;
        // for (a, b) in self.packet[0..data.len()].iter_mut().zip(data.iter()) { *a = *b; }

        // enable "disabled" interrupt
        self.radio.intenset.write(|w| { 
            w.disabled().bit(true)
        });
        // set packet pointer
        self.radio.packetptr.write(|w| unsafe { w.packetptr().bits((&self.packet as *const u8) as u32) });
        // start transmission task
        self.radio.tasks_txen.write(|w| { w.tasks_txen().bit(true) });
    }

    pub fn init_reception(&self) {
        // POWER
        // 1 (default)
        self.radio.power.write(|w| { w.power().enabled() });

        // MODE
        // MODE: data rate and modulation
        self.radio.mode.write(|w| { w.mode().ble_1mbit() });

        // FREQUENCY
        // FREQUENCY: [0..100] freq = 2400 MHz + freq => 90
        self.radio.frequency.write(|w| unsafe { w.frequency().bits(90) });

        // PCNF0
        // LFLEN: length field length in bits => 8
        // S0LEN: S0 length in bytes => 0 (default)
        // S1LEN: S1 length in bits => 0 (default)
        // S1INCL: 0 (default)
        // PLEN: 0 (default)
        self.radio.pcnf0.write(|w| unsafe {
            w.lflen().bits(8)
             .plen().bit(true)
        });

        // PCNF1
        // MAXLEN: max length of payload packet => 255
        // STATLEN: 0 (default)
        // BALEN: base address length => 4
        // ENDIAN: 0 (default) => 1
        // WHITEEN: 0 (default)
        self.radio.pcnf1.write(|w| unsafe {
            w.maxlen().bits(255)
             .balen().bits(4)
             .endian().bit(true)
        });

        // BASE0
        // BASE0: 0xABCDABCD
        self.radio.base0.write(|w| unsafe { w.base0().bits(0xABCDABCD) });
        self.radio.base1.write(|w| unsafe { w.base1().bits(0xABCDABCD) });

        // PREFIX0
        // AP0: 0xDA
        self.radio.prefix0.write(|w| unsafe {
            w.ap0().bits(0xDA)
             .ap1().bits(0xEF)
        });

        // DAB[0]
        // DAB
        self.radio.dab[0].write(|w| unsafe { w.dab().bits(0xABCDABCD) });

        // DAB[1]
        // DAB
        self.radio.dab[1].write(|w| unsafe { w.dab().bits(0xABCDABCD) });

        // DAP[0]
        // DAP
        self.radio.dap[0].write(|w| unsafe { w.dap().bits(0xDA) });

        // DAP[1]
        // DAP
        self.radio.dap[1].write(|w| unsafe { w.dap().bits(0xDA) });

        // DACNF
        // ENA0, ENA1 => true
        self.radio.dacnf.write(|w| { w.ena0().bit(true) });
        // self.radio.dacnf.write(|w| { w.ena0().bit(true).ena1().bit(true) });

        // TXADDRESS
        // TXADDRESS: 0 (default)

        // RXADDRESSES
        // ADDR0, ADDR1
        self.radio.rxaddresses.write(|w| { w.addr0().bit(true).addr1().bit(true) });

        // CRCCNF
        // LEN: length => 3
        // SKIPADDR: 0 (default)
        self.radio.crccnf.write(|w| w.len().bits(3));

        // CRCPOLY
        // x24 + x10 + x9 + x6 + x4 + x3 + x + 1
        // CRCPOLY: 00000000_00000110_01011011
        self.radio.crcpoly.write(|w| unsafe { w.bits(0b00000000_00000110_01011011) });

        // Shortcuts
        // READY - START
        // ADDRESS - RSSISTART
        self.radio.shorts.write(|w| {
            w.ready_start().bit(true)
             .address_rssistart().bit(true)
             .end_disable().bit(true)
        });
    }

    pub fn start_reception(&self) {
        self.radio.intenset.write(|w| { 
            w
            //  .ready().bit(true)
            //  .address().bit(true)
            //  .payload().bit(true)
            //  .end().bit(true)
             .disabled().bit(true)
            //  .devmiss().bit(true)
        });
        // set packet pointer
        self.radio.packetptr.write(|w| unsafe { w.packetptr().bits((&self.packet as *const u8) as u32) });

        self.radio.tasks_rxen.write(|w| { w.tasks_rxen().bit(true) });
    }

    pub fn is_ready_set(&self) -> bool {
        self.radio.intenset.read().ready().is_enabled()
    }

    pub fn state(&self) -> nrf52810_pac::generic::Variant<u8, nrf52810_pac::radio::state::STATE_A> {
        self.radio.state.read().state().variant()
    }

    pub fn is_ready(&self) -> bool {
        self.radio.events_ready.read().events_ready().variant() == nrf52810_pac::radio::events_ready::EVENTS_READY_A::GENERATED
    }

    pub fn clear_ready(&self) {
        self.radio.intenclr.write(|w| { w.ready().bit(true) });
    }

    pub fn is_address(&self) -> bool {
        self.radio.events_address.read().events_address().variant() == nrf52810_pac::radio::events_address::EVENTS_ADDRESS_A::GENERATED
    }

    pub fn clear_address(&self) {
        self.radio.intenclr.write(|w| { w.address().bit(true) });
    }

    pub fn is_payload(&self) -> bool {
        self.radio.events_payload.read().events_payload().variant() == nrf52810_pac::radio::events_payload::EVENTS_PAYLOAD_A::GENERATED
    }

    pub fn clear_payload(&self) {
        self.radio.intenclr.write(|w| { w.payload().bit(true) });
    }

    pub fn start_rssi(&self) {
        self.radio.tasks_rssistart.write(|w| { w.tasks_rssistart().bit(true) });
    }

    pub fn stop_rssi(&self) {
        self.radio.tasks_rssistop.write(|w| { w.tasks_rssistop().bit(true) });
    }

    pub fn rssi(&self) -> u8 {
        self.radio.rssisample.read().rssisample().bits()
    }

    pub fn is_rssi_ready(&self) -> bool {
        self.radio.events_rssiend.read().events_rssiend().is_generated()
    }

    pub fn clear_all(&self) {
        self.radio.intenclr.write(|w| {
            w.ready().bit(true)
             .address().bit(true)
             .payload().bit(true)
             .end().bit(true)
             .disabled().bit(true)
             .devmatch().bit(true)
             .devmiss().bit(true)
             .rssiend().bit(true)
             .bcmatch().bit(true)
             .crcok().bit(true)
             .crcerror().bit(true)
        });
    }

    pub fn set_all(&self) {
        self.radio.intenset.write(|w| {
            w.ready().bit(true)
             .address().bit(true)
             .payload().bit(true)
             .end().bit(true)
             .disabled().bit(true)
             .devmatch().bit(true)
             .devmiss().bit(true)
             .rssiend().bit(true)
             .bcmatch().bit(true)
             .crcok().bit(true)
             .crcerror().bit(true)
        });
    }
  
    pub fn event_ready(&self) -> bool {
        self.radio.events_ready.read().events_ready().is_generated()
    }

    pub fn event_address(&self) -> bool {
        self.radio.events_address.read().events_address().is_generated()
    }

    pub fn event_payload(&self) -> bool {
        self.radio.events_payload.read().events_payload().is_generated()
    }

    pub fn event_end(&self) -> bool {
        self.radio.events_end.read().events_end().is_generated()
    }

    pub fn event_disabled(&self) -> bool {
        self.radio.events_disabled.read().events_disabled().is_generated()
    }

    pub fn event_devmatch(&self) -> bool {
        self.radio.events_devmatch.read().events_devmatch().is_generated()
    }

    pub fn event_devmiss(&self) -> bool {
        self.radio.events_devmiss.read().events_devmiss().is_generated()
    }

    pub fn event_rssiend(&self) -> bool {
        self.radio.events_rssiend.read().events_rssiend().is_generated()
    }

    pub fn event_bcmatch(&self) -> bool {
        self.radio.events_bcmatch.read().events_bcmatch().is_generated()
    }

    pub fn event_crcok(&self) -> bool {
        self.radio.events_crcok.read().events_crcok().is_generated()
    }

    pub fn event_crcerror(&self) -> bool {
        self.radio.events_crcerror.read().events_crcerror().is_generated()
    }

    pub fn event_reset_all(&self) {
        self.radio.events_ready.write(|w| { w.events_ready().not_generated() });
        self.radio.events_address.write(|w| { w.events_address().not_generated() });
        self.radio.events_payload.write(|w| { w.events_payload().not_generated() });
        self.radio.events_end.write(|w| { w.events_end().not_generated() });
        self.radio.events_disabled.write(|w| { w.events_disabled().not_generated() });
        self.radio.events_devmatch.write(|w| { w.events_devmatch().not_generated() });
        self.radio.events_devmiss.write(|w| { w.events_devmiss().not_generated() });
        self.radio.events_bcmatch.write(|w| { w.events_bcmatch().not_generated() });
        self.radio.events_crcok.write(|w| { w.events_crcok().not_generated() });
        self.radio.events_crcerror.write(|w| { w.events_crcerror().not_generated() });
    }

    pub fn address_match(&self) -> u8 {
        self.radio.rxmatch.read().rxmatch().bits()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        if self.packet[0] > 0 {
            Some(&self.packet[1..self.packet[0] as usize])
        } else {
            None
        }
    }
}
