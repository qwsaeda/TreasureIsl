/* ea:   0x00065404 */
/* name: sub_65404 */
/* size: 456 */
int __fastcall sub_65404(int a1, _WORD *a2)
{
  int v4; // r0
  unsigned __int16 *v5; // r4
  int v6; // r4
  void *v8; // r0
  int v9; // r4
  int v10; // r0
  _BYTE v11[3]; // [sp+1h] [bp-53h] BYREF
  int v12; // [sp+4h] [bp-50h] BYREF
  _BYTE v13[8]; // [sp+8h] [bp-4Ch] BYREF
  _DWORD v14[2]; // [sp+10h] [bp-44h] BYREF
  _BYTE v15[8]; // [sp+18h] [bp-3Ch] BYREF
  unsigned __int16 *v16; // [sp+20h] [bp-34h] BYREF
  int v17; // [sp+24h] [bp-30h]
  void *v18[9]; // [sp+30h] [bp-24h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  sub_3E0A4(&v16);
  v4 = sub_12F04(a2);
  sub_3E0B8(&v16, a2, &a2[v4]);
  v5 = v16;
  if ( v16[((v17 - (int)v16) >> 1) - 1] == 92 )
  {
    sub_65A74((int)&v16, (void *)(v17 - 2));
    v5 = v16;
  }
  v12 = sub_18094(v5);
  v6 = *(_DWORD *)sub_65E30(a1 + 44, v13);
  if ( *(_DWORD *)sub_66464(a1 + 44, v13, &v12) != v6 )
  {
    sub_132A0((void **)&v16);
    return 1;
  }
  v8 = operator new(0x30u);
  if ( v8 == nullptr )
    goto LABEL_11;
  v9 = sub_1E61C((int)v8);
  if ( v9 == 0 )
    goto LABEL_11;
  sub_132FC(v18, (_DWORD *)(a1 + 4));
  sub_66658(v18, v16, v17, v11);
  sub_162DC(v18, (char *)aLpk);
  if ( (*(unsigned __int8 (__fastcall **)(int, void *))(*(_DWORD *)v9 + 4))(v9, v18[0]) == 0 )
  {
    (**(void (__fastcall ***)(int, int))v9)(v9, 1);
    sub_12F48(v18);
LABEL_11:
    sub_12F48((void **)&v16);
    return 0;
  }
  v14[1] = v9;
  v14[0] = v12;
  v10 = sub_66324(a1 + 44, v13, v14);
  sub_65E88(v15, v10);
  sub_12F48(v18);
  sub_12EA0((int)&v16);
  return 1;
}

