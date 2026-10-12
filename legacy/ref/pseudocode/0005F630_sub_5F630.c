/* ea:   0x0005F630 */
/* name: sub_5F630 */
/* size: 740 */
int __fastcall sub_5F630(int a1, _WORD *a2, _WORD *a3, int a4)
{
  int v8; // r0
  int v9; // r5
  int v10; // r0
  int v11; // r5
  int v12; // r4
  int v14; // r0
  int v15; // r0
  int v16; // r0
  _DWORD *v17; // r0
  int v18; // r0
  int v19; // r0
  _BYTE v20[8]; // [sp+8h] [bp-1BCh] BYREF
  void *v21[4]; // [sp+10h] [bp-1B4h] BYREF
  void *v22[4]; // [sp+20h] [bp-1A4h] BYREF
  void *v23[4]; // [sp+30h] [bp-194h] BYREF
  _DWORD v24[4]; // [sp+40h] [bp-184h] BYREF
  _DWORD v25[4]; // [sp+50h] [bp-174h] BYREF
  void *v26[4]; // [sp+60h] [bp-164h] BYREF
  int v27; // [sp+70h] [bp-154h] BYREF
  _DWORD v28[3]; // [sp+74h] [bp-150h] BYREF
  int v29; // [sp+80h] [bp-144h] BYREF
  _DWORD v30[3]; // [sp+84h] [bp-140h] BYREF
  int v31; // [sp+90h] [bp-134h] BYREF
  _DWORD v32[3]; // [sp+94h] [bp-130h] BYREF
  _BYTE v33[4]; // [sp+A0h] [bp-124h] BYREF
  _BYTE v34[12]; // [sp+A4h] [bp-120h] BYREF
  _BYTE v35[16]; // [sp+B0h] [bp-114h] BYREF
  _DWORD v36[4]; // [sp+C0h] [bp-104h] BYREF
  _BYTE v37[4]; // [sp+D0h] [bp-F4h] BYREF
  _BYTE v38[12]; // [sp+D4h] [bp-F0h] BYREF
  int v39; // [sp+E0h] [bp-E4h]
  _BYTE v40[4]; // [sp+E8h] [bp-DCh] BYREF
  _BYTE v41[12]; // [sp+ECh] [bp-D8h] BYREF
  _BYTE v42[8]; // [sp+F8h] [bp-CCh] BYREF
  _BYTE v43[32]; // [sp+100h] [bp-C4h] BYREF
  wchar_t v44[82]; // [sp+120h] [bp-A4h] BYREF

  sub_3E0A4(v22);
  v8 = sub_12F04(a2);
  sub_3E0B8(v22, a2, &a2[v8]);
  v31 = sub_19A4C((_WORD *)v22[0]);
  sub_132FC(v32, v22);
  v9 = *(_DWORD *)sub_6157C(a1 + 4, v20, &v31);
  sub_12EA0((int)v32);
  sub_12F48(v22);
  if ( v9 == *(_DWORD *)sub_603A4(a1 + 4, v20) )
  {
    sub_3E0A4(v26);
    v16 = sub_12F04(a2);
    sub_3E0B8(v26, a2, &a2[v16]);
    sub_5FF10(a1, v37, v26);
    sub_12F48(v26);
    sub_1333C(v25);
    sub_605D4((int)v25, a4);
    v17 = sub_16480(v36, a3);
    v18 = sub_605A4(v40, v17);
    v19 = sub_60290(v43, v18, v25);
    sub_60118(v39, v20, v19);
    sub_19A98((int)v43);
    sub_12EA0((int)v41);
    sub_12EA0((int)v36);
    sub_12EA0((int)v25);
    sub_12EA0((int)v38);
    return 1;
  }
  else
  {
    sub_3E0A4(v21);
    v10 = sub_12F04(a3);
    sub_3E0B8(v21, a3, &a3[v10]);
    v27 = sub_19A4C((_WORD *)v21[0]);
    sub_132FC(v28, v21);
    v11 = v9 + 32;
    v12 = *(_DWORD *)sub_61560(v11, v20, &v27);
    sub_12EA0((int)v28);
    sub_12F48(v21);
    if ( v12 == *(_DWORD *)sub_60368(v11, v20) )
    {
      sub_1333C(v24);
      itow(a4, v44, 10);
      sub_60070(v24, v44);
      sub_3E0A4(v23);
      v14 = sub_12F04(a3);
      sub_3E0B8(v23, a3, &a3[v14]);
      v29 = sub_19A4C((_WORD *)v23[0]);
      sub_132FC(v30, v23);
      sub_602C4(v33, &v29);
      sub_60614(v35, v24);
      v15 = sub_60C54(v11, v20, v33);
      sub_603B4(v42, v15);
      sub_12EA0((int)v35);
      sub_12EA0((int)v34);
      sub_12EA0((int)v30);
      sub_12F48(v23);
      sub_12EA0((int)v24);
    }
    else
    {
      itow(a4, v44, 10);
      sub_60070(v12 + 32, v44);
    }
    return 1;
  }
}

