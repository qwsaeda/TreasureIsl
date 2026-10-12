/* ea:   0x0001EECC */
/* name: sub_1EECC */
/* size: 384 */
int __fastcall sub_1EECC(int a1, int a2, int a3)
{
  char v7; // [sp+0h] [bp-1Ch] BYREF
  char v8; // [sp+1h] [bp-1Bh] BYREF
  char v9; // [sp+2h] [bp-1Ah] BYREF
  char v10; // [sp+3h] [bp-19h] BYREF
  char v11; // [sp+4h] [bp-18h] BYREF
  char v12; // [sp+5h] [bp-17h] BYREF
  char v13; // [sp+6h] [bp-16h] BYREF

  sub_4EC88();
  *(_BYTE *)(a1 + 48) = 0;
  *(_BYTE *)(a1 + 49) = 0;
  sub_53640(a1 + 52);
  sub_30B10(a1 + 64, &v7);
  sub_30B10(a1 + 68, &v8);
  sub_32EC4(a1 + 72);
  sub_31A44(a1 + 144);
  sub_39E80(a1 + 192);
  *(_DWORD *)(a1 + 264) = 0;
  *(_DWORD *)(a1 + 268) = 0;
  sub_3326C(a1 + 272, &v9, 0);
  *(_DWORD *)(a1 + 276) = 0;
  *(_DWORD *)(a1 + 280) = 0;
  sub_324D4(a1 + 284, &v10, 0);
  sub_4B41C(a1 + 288);
  sub_30BD8(a1 + 496, &v12, &v11);
  sub_2F5F0(a1 + 508);
  sub_2F920(a1 + 520);
  sub_2FA34(a1 + 532);
  sub_2FAF0(a1 + 552);
  sub_2FAF0(a1 + 564);
  *(_DWORD *)(a1 + 576) = 0;
  *(_DWORD *)(a1 + 580) = 0;
  *(_DWORD *)(a1 + 584) = 0;
  *(_DWORD *)(a1 + 588) = 0;
  *(_DWORD *)(a1 + 592) = 0;
  *(_DWORD *)(a1 + 596) = 0;
  *(_DWORD *)(a1 + 600) = 0;
  *(_DWORD *)(a1 + 604) = a2;
  sub_5398C(a1 + 608, a1);
  sub_1F074(a1 + 652);
  sub_2FC1C(a1 + 676, &v13);
  *(_DWORD *)(a1 + 672) = &off_6DAF4;
  *(_DWORD *)(a1 + 688) = a3;
  *(_DWORD *)(a1 + 692) = 0;
  *(_DWORD *)(a1 + 696) = 0;
  *(_DWORD *)(a1 + 700) = 0;
  *(_DWORD *)(a1 + 704) = 0;
  *(_DWORD *)(a1 + 708) = 0;
  *(_DWORD *)(a1 + 712) = 1;
  *(_DWORD *)(a1 + 716) = 27;
  *(_DWORD *)(a1 + 720) = 72;
  *(_DWORD *)(a1 + 724) = 0;
  *(_DWORD *)(a1 + 728) = 0;
  *(_DWORD *)(a1 + 732) = 0;
  sub_1333C((_DWORD *)(a1 + 736));
  *(_DWORD *)a1 = off_6DAD0;
  return a1;
}

