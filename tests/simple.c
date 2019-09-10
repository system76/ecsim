#include "include/signature.h"

void reset() {
    __asm__("ljmp 0");
}

#include "include/gpio.h"

void gpio_init() {
    // Enable LPC reset on GPD2
    GCR = 0x04;

    // Set GPIO data
    GPDRA = 0;
    GPDRB = (1 << 0);
    GPDRC = 0;
    GPDRD = (1 << 5) | (1 << 4) | (1 << 3);
    GPDRE = 0;
    GPDRF = (1 << 7) | (1 << 6);
    GPDRG = 0;
    GPDRH = 0;
    GPDRI = 0;
    GPDRJ = 0;

    // Set GPIO control
    GPCRA0 = 0x80;
    GPCRA1 = 0x00;
    GPCRA2 = 0x00;
    GPCRA3 = 0x80;
    GPCRA4 = 0x40;
    GPCRA5 = 0x44;
    GPCRA6 = 0x44;
    GPCRA7 = 0x44;
    GPCRB0 = 0x44;
    GPCRB1 = 0x44;
    GPCRB2 = 0x84;
    GPCRB3 = 0x00;
    GPCRB4 = 0x00;
    GPCRB5 = 0x44;
    GPCRB6 = 0x84;
    GPCRB7 = 0x80;
    GPCRC0 = 0x80;
    GPCRC1 = 0x84;
    GPCRC2 = 0x84;
    GPCRC3 = 0x84;
    GPCRC4 = 0x44;
    GPCRC5 = 0x44;
    GPCRC6 = 0x40;
    GPCRC7 = 0x44;
    GPCRD0 = 0x84;
    GPCRD1 = 0x84;
    GPCRD2 = 0x00;
    GPCRD3 = 0x80;
    GPCRD4 = 0x80;
    GPCRD5 = 0x44;
    GPCRD6 = 0x80;
    GPCRD7 = 0x80;
    GPCRE0 = 0x44;
    GPCRE1 = 0x44;
    GPCRE2 = 0x80;
    GPCRE3 = 0x40;
    GPCRE4 = 0x42;
    GPCRE5 = 0x40;
    GPCRE6 = 0x44;
    GPCRE7 = 0x44;
    GPCRF0 = 0x80;
    GPCRF1 = 0x44;
    GPCRF2 = 0x84;
    GPCRF3 = 0x44;
    GPCRF4 = 0x80;
    GPCRF5 = 0x80;
    GPCRF6 = 0x00;
    GPCRF7 = 0x80;
    GPCRG0 = 0x44;
    GPCRG1 = 0x44;
    GPCRG2 = 0x40;
    GPCRG3 = 0x00;
    GPCRG4 = 0x00;
    GPCRG5 = 0x00;
    GPCRG6 = 0x44;
    GPCRG7 = 0x00;
    GPCRH0 = 0x00;
    GPCRH1 = 0x80;
    GPCRH2 = 0x44;
    GPCRH3 = 0x44;
    GPCRH4 = 0x80;
    GPCRH5 = 0x80;
    GPCRH6 = 0x80;
    GPCRH7 = 0x80;
    GPCRI0 = 0x00;
    GPCRI1 = 0x00;
    GPCRI2 = 0x80;
    GPCRI3 = 0x00;
    GPCRI4 = 0x00;
    GPCRI5 = 0x80;
    GPCRI6 = 0x80;
    GPCRI7 = 0x80;
    GPCRJ0 = 0x82;
    GPCRJ1 = 0x80;
    GPCRJ2 = 0x40;
    GPCRJ3 = 0x80;
    GPCRJ4 = 0x44;
    GPCRJ5 = 0x40;
    GPCRJ6 = 0x44;
    GPCRJ7 = 0x80;
}

#include "include/gctrl.h"

void gctrl_init() {
    SPCTRL1 = 0x03;
    BADRSEL = 0;
    RSTS = 0x84;
}

#include "include/kbc.h"

void kbc_init() {
    KBIRQR = 0;
    KBHICR = 0x48;
}

#include "include/pmc.h"

void pmc_init() {
    PM1CTL = 0x41;
    PM2CTL = 0x41;
}

#include "include/ps2.h"

void ps2_init() {
    PSCTL1 = 0x11;
    PSCTL2 = 0x41;
    PSCTL3 = 0x41;
    PSINT1 = 0x04;
    PSINT2 = 0x04;
    PSINT3 = 0x04;
}

#include "include/kbscan.h"

void kbscan_init() {
    KSOCTRL = 0x05;
    KSICTRLR = 0x04;
}

void main() {
    gpio_init();

    gctrl_init();

    kbc_init();

    pmc_init();

    kbscan_init();

    //TODO: INTC, PECI, PWM, SMBUS

    GPDRA |= (1 << 7);

    for(;;) {}
}
