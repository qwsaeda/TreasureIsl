/* ea:   0x0005F914 */
/* name: sub_5F914 */
/* size: 728 */
int __fastcall sub_5F914(int a1, _WORD *a2, _WORD *a3, int a4)
{
  int v8; // r0
  int v9; // r5
  int v10; // r0
  int v11; // r5
  int v12; // r4
  int v14; // r0
  int v15; // r0
  int v16; // r0
  int v17; // r0
  int v18; // r0
  int v19; // r0
  _BYTE v20[8]; // [sp+8h] [bp-138h] BYREF
  void *v21[4]; // [sp+10h] [bp-130h] BYREF
  void *v22[4]; // [sp+20h] [bp-120h] BYREF
  void *v23[4]; // [sp+30h] [bp-110h] BYREF
  _DWORD v24[4]; // [sp+40h] [bp-100h] BYREF
  void *v25[4]; // [sp+50h] [bp-F0h] BYREF
  _DWORD v26[4]; // [sp+60h] [bp-E0h] BYREF
  _DWORD v27[4]; // [sp+70h] [bp-D0h] BYREF
  int v28; // [sp+80h] [bp-C0h] BYREF
  _DWORD v29[3]; // [sp+84h] [bp-BCh] BYREF
  int v30; // [sp+90h] [bp-B0h] BYREF
  _DWORD v31[3]; // [sp+94h] [bp-ACh] BYREF
  int v32; // [sp+A0h] [bp-A0h] BYREF
  _DWORD v33[3]; // [sp+A4h] [bp-9Ch] BYREF
  _BYTE v34[4]; // [sp+B0h] [bp-90h] BYREF
  _BYTE v35[12]; // [sp+B4h] [bp-8Ch] BYREF
  _BYTE v36[16]; // [sp+C0h] [bp-80h] BYREF
  _BYTE v37[4]; // [sp+D0h] [bp-70h] BYREF
  _BYTE v38[12]; // [sp+D4h] [bp-6Ch] BYREF
  int v39; // [sp+E0h] [bp-60h]
  _BYTE v40[4]; // [sp+E8h] [bp-58h] BYREF
  _BYTE v41[12]; // [sp+ECh] [bp-54h] BYREF
  _BYTE v42[8]; // [sp+F8h] [bp-48h] BYREF
  _BYTE v43[64]; // [sp+100h] [bp-40h] BYREF

  sub_3E0A4(v22);
  v8 = sub_12F04(a2);
  sub_3E0B8(v22, a2, &a2[v8]);
  v32 = sub_19A4C((_WORD *)v22[0]);
  sub_132FC(v33, v22);
  v9 = *(_DWORD *)sub_6157C(a1 + 4, v20, &v32);
  sub_12EA0((int)v33);
  sub_12F48(v22);
  if ( v9 == *(_DWORD *)sub_603A4(a1 + 4, v20) )
  {
    sub_3E0A4(v25);
    v16 = sub_12F04(a2);
    sub_3E0B8(v25, a2, &a2[v16]);
    sub_5FF10(a1, v37, v25);
    sub_12F48(v25);
    sub_1333C(v26);
    sub_161B0((int)v26, a4);
    sub_3E0A4(v27);
    v17 = sub_12F04(a3);
    sub_3E0B8(v27, a3, &a3[v17]);
    v18 = sub_605A4(v40, v27);
    v19 = sub_60290(v43, v18, v26);
    sub_60118(v39, v20, v19);
    sub_19A98((int)v43);
    sub_12EA0((int)v41);
    sub_12EA0((int)v27);
    sub_12EA0((int)v26);
    sub_12EA0((int)v38);
    return 1;
  }
  else
  {
    sub_3E0A4(v21);
    v10 = sub_12F04(a3);
    sub_3E0B8(v21, a3, &a3[v10]);
    v28 = sub_19A4C((_WORD *)v21[0]);
    sub_132FC(v29, v21);
    v11 = v9 + 32;
    v12 = *(_DWORD *)sub_61560(v11, v20, &v28);
    sub_12EA0((int)v29);
    sub_12F48(v21);
    if ( v12 == *(_DWORD *)sub_60368(v11, v20) )
    {
      sub_1333C(v24);
      sub_161B0((int)v24, a4);
      sub_3E0A4(v23);
      v14 = sub_12F04(a3);
      sub_3E0B8(v23, a3, &a3[v14]);
      v30 = sub_19A4C((_WORD *)v23[0]);
      sub_132FC(v31, v23);
      sub_602C4(v34, &v30);
      sub_60614(v36, v24);
      v15 = sub_60C54(v11, v20, v34);
      sub_603B4(v42, v15);
      sub_12EA0((int)v36);
      sub_12EA0((int)v35);
      sub_12EA0((int)v31);
      sub_12F48(v23);
      sub_12EA0((int)v24);
    }
    else
    {
      sub_161B0(v12 + 32, a4);
    }
    return 1;
  }
}

