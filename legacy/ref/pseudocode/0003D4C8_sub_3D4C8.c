/* ea:   0x0003D4C8 */
/* name: sub_3D4C8 */
/* size: 1040 */
int __fastcall sub_3D4C8(int a1, int a2)
{
  int v4; // r0
  int v5; // r0
  _DWORD *v7; // r0
  _DWORD *v8; // r5
  int v9; // r4
  int v10; // r6
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r0
  int v15; // r0
  int v16; // r0
  void *v17; // r0
  int v18; // r0
  _DWORD *v19; // r0
  void *v20; // r0
  char v21; // [sp+4h] [bp-8Ch] BYREF
  char v22; // [sp+5h] [bp-8Bh] BYREF
  char v23; // [sp+6h] [bp-8Ah] BYREF
  char v24; // [sp+7h] [bp-89h] BYREF
  void *v25[4]; // [sp+Ch] [bp-84h] BYREF
  _DWORD v26[4]; // [sp+1Ch] [bp-74h] BYREF
  void *v27[4]; // [sp+2Ch] [bp-64h] BYREF
  void *v28[4]; // [sp+3Ch] [bp-54h] BYREF
  void *v29[4]; // [sp+4Ch] [bp-44h] BYREF
  _BYTE v30[16]; // [sp+5Ch] [bp-34h] BYREF
  struct _SYSTEMTIME SystemTime; // [sp+6Ch] [bp-24h] BYREF

  sub_3EB24();
  GetLocalTime(&SystemTime);
  srand(SystemTime.wMinute + SystemTime.wSecond + SystemTime.wMilliseconds);
  v4 = sub_6221C(1, a2);
  *(_DWORD *)(a1 + 4) = v4;
  if ( v4 == 0 )
    return 0;
  sub_1333C(v25);
  sub_1309C((int)v25);
  sub_162DC(v25, (char *)aData);
  v5 = sub_65144(v25[0]);
  *(_DWORD *)(a1 + 20) = v5;
  if ( v5 == 0 )
  {
    sub_12F48(v25);
    return 0;
  }
  (*(void (__fastcall **)(int, wchar_t *))(*(_DWORD *)v5 + 4))(v5, aCards);
  (*(void (__fastcall **)(_DWORD, wchar_t *))(**(_DWORD **)(a1 + 20) + 4))(*(_DWORD *)(a1 + 20), aGoods);
  (*(void (__fastcall **)(_DWORD, wchar_t *))(**(_DWORD **)(a1 + 20) + 4))(*(_DWORD *)(a1 + 20), aMusic);
  v7 = operator new(0x18u);
  v8 = v7;
  if ( v7 != nullptr )
  {
    v9 = *(_DWORD *)(a1 + 20);
    sub_3DFA8(v7);
    v8[4] = v9;
    v10 = 0;
    v8[3] = 0;
    v8[5] = 0;
  }
  else
  {
    v10 = 0;
    v8 = nullptr;
  }
  *(_DWORD *)(a1 + 28) = v8;
  if ( v8 == nullptr )
  {
    sub_12F48(v25);
    return 0;
  }
  sub_3E698(v8);
  v11 = sub_12F04(aMusicGlmusicMo);
  sub_1318C((void **)(a1 + 36), (char *)aMusicGlmusicMo, (int)&aMusicGlmusicMo[v11]);
  sub_3EA28(*(_DWORD *)(a1 + 28), 255);
  if ( *(_BYTE *)(a1 + 32) != 0 )
    sub_3E758(*(_DWORD *)(a1 + 28), *(_DWORD *)(a1 + 36));
  sub_3E0A4(v27, &v21);
  v12 = sub_12F04(aMusicPressWav);
  sub_3E0B8(v27, aMusicPressWav, &aMusicPressWav[v12]);
  sub_3DFF0(a1 + 48, v27);
  sub_12F48(v27);
  sub_3E0A4(v28, &v22);
  v13 = sub_12F04(aMusicInvalidWa);
  sub_3E0B8(v28, aMusicInvalidWa, &aMusicInvalidWa[v13]);
  sub_3DFF0(a1 + 48, v28);
  sub_12F48(v28);
  sub_3E0A4(v29, &v23);
  v14 = sub_12F04(aMusicAttackWav);
  sub_3E0B8(v29, aMusicAttackWav, &aMusicAttackWav[v14]);
  sub_3DFF0(a1 + 48, v29);
  sub_12F48(v29);
  sub_3E0A4(v30, &v24);
  v15 = sub_12F04(aMusicLevelupWa);
  sub_3E0B8(v30, aMusicLevelupWa, &aMusicLevelupWa[v15]);
  sub_3DFF0(a1 + 48, v30);
  sub_12EA0((int)v30);
  sub_16480(&SystemTime, aMusicVictoryWa);
  sub_3DFF0(a1 + 48, &SystemTime);
  sub_12EA0((int)&SystemTime);
  if ( operator new(0x14u) != nullptr )
    v16 = sub_3E0E0();
  else
    v16 = 0;
  *(_DWORD *)(a1 + 24) = v16;
  if ( v16 == 0 || (unsigned __int8)sub_3E194(v16, *(_DWORD *)(a1 + 20)) == 0 )
    goto LABEL_23;
  sub_1333C(v26);
  v17 = operator new(0x3FCu);
  if ( v17 != nullptr )
    v18 = sub_43B8C(v17, *(_DWORD *)(a1 + 4));
  else
    v18 = 0;
  *(_DWORD *)(a1 + 16) = v18;
  if ( v18 == 0
    || (sub_1309C((int)v26),
        sub_162DC(v26, (char *)aDataUiPak),
        v19 = sub_132FC(&SystemTime, v26),
        sub_43D94(*(_DWORD *)(a1 + 16), *v19, v19[1], v19[2]),
        sub_3DC04(a1, 2),
        *(_DWORD *)(a1 + 8) == 0) )
  {
    sub_12EA0((int)v26);
LABEL_23:
    sub_12EA0((int)v25);
    return 0;
  }
  sub_1309C((int)v26);
  sub_162DC(v26, (char *)aDataGlpicsPak);
  v20 = operator new(0x18u);
  if ( v20 != nullptr )
    v10 = sub_4290C(v20, v26[0], *(_DWORD *)(a1 + 4), a1, *(_DWORD *)(a1 + 24));
  *(_DWORD *)(a1 + 60) = v10;
  if ( v10 != 0 )
    sub_42AE8(v10);
  sub_12EA0((int)v26);
  sub_12EA0((int)v25);
  return 1;
}

