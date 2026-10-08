/* ea:   0x000655CC */
/* name: sub_655CC */
/* size: 964 */
int __fastcall sub_655CC(int a1, char *a2, int a3, int a4)
{
  int v8; // r5
  int v10; // r0
  char v11; // r6
  bool v12; // r4
  unsigned __int16 *v13; // r3
  int v14; // r5
  int v15; // r6
  int v16; // r0
  int v17; // r0
  int v18; // r0
  int v19; // r4
  int v20; // r0
  int v21; // [sp+4h] [bp-8Ch] BYREF
  _BYTE v22[8]; // [sp+8h] [bp-88h] BYREF
  unsigned __int16 *v23; // [sp+10h] [bp-80h] BYREF
  int v24; // [sp+14h] [bp-7Ch]
  void *v25[4]; // [sp+20h] [bp-70h] BYREF
  _DWORD v26[4]; // [sp+30h] [bp-60h] BYREF
  void *v27[4]; // [sp+40h] [bp-50h] BYREF
  _DWORD v28[4]; // [sp+50h] [bp-40h] BYREF
  _DWORD v29[12]; // [sp+60h] [bp-30h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  if ( (unsigned __int8)sub_6693C(a1, a2) != 0 )
  {
    v8 = sub_66A04(a1);
    if ( v8 == 0 )
      return 0;
    sub_1333C(v25);
    if ( a2 == nullptr )
      __debugbreak();
    sub_161B0((int)v25, a1 + 4);
    sub_162DC(v25, a2);
    if ( (*(unsigned __int8 (__fastcall **)(int, void *, int, int))(*(_DWORD *)(v8 + 4) + 4))(v8 + 4, v25[0], a3, a4) == 0 )
    {
      (*(void (__fastcall **)(int, int))(*(_DWORD *)a1 + 12))(a1, v8);
      sub_12F48(v25);
      return 0;
    }
    sub_12F48(v25);
    return v8;
  }
  else
  {
    sub_1333C(&v23);
    sub_3E0A4(v27);
    v10 = sub_12F04(a2);
    sub_3E0B8(v27, a2, &a2[2 * v10]);
    v11 = 1;
    v12 = (unsigned __int8)sub_65F78(v27, &v23) == 0;
    sub_12F48(v27);
    if ( v12 )
      goto LABEL_18;
    v13 = v23;
    if ( v23[((v24 - (int)v23) >> 1) - 1] == 92 )
    {
      sub_65B94(v24 - 2, v24, 1);
      v13 = v23;
      v24 -= 2;
    }
    v21 = sub_18094(v13);
    v14 = *(_DWORD *)sub_66464(a1 + 44, v22, &v21);
    if ( v14 == *(_DWORD *)sub_65E30(a1 + 44, v22) )
      v11 = 0;
    if ( v11 == 0 )
    {
      v19 = sub_66A04(a1);
      if ( v19 != 0 )
      {
        sub_1333C(v29);
        v20 = sub_66998(a1, (int)v29);
        if ( (unsigned __int8)sub_667FC(v19, v20, a3, a4) != 0 )
        {
          sub_12EA0((int)v29);
          sub_12EA0((int)&v23);
          return v19;
        }
        (*(void (__fastcall **)(int, int))(*(_DWORD *)a1 + 12))(a1, v19);
        sub_12EA0((int)v29);
      }
      sub_12EA0((int)&v23);
      return 0;
    }
    v15 = sub_66A84(a1);
    if ( v15 == 0 )
    {
LABEL_18:
      sub_12F48((void **)&v23);
      return 0;
    }
    sub_1333C(v26);
    sub_3E0A4(v28);
    v16 = sub_12F04(a2);
    sub_3E0B8(v28, a2, &a2[2 * v16]);
    sub_65F0C(v28, v26);
    sub_12EA0((int)v28);
    v17 = (*(int (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(v14 + 20) + 12))(*(_DWORD *)(v14 + 20), v26[0]);
    v18 = (*(int (__fastcall **)(_DWORD, int))(**(_DWORD **)(v14 + 20) + 16))(*(_DWORD *)(v14 + 20), v17);
    if ( v18 != 0 && (unsigned __int8)sub_64EF8(v15, *(_DWORD *)(v14 + 20), v18) != 0 )
    {
      sub_12EA0((int)v26);
      sub_12EA0((int)&v23);
      return v15;
    }
    else
    {
      (*(void (__fastcall **)(int, int))(*(_DWORD *)a1 + 12))(a1, v15);
      sub_12EA0((int)v26);
      sub_12EA0((int)&v23);
      return 0;
    }
  }
}

