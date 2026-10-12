/* ea:   0x00053D48 */
/* name: sub_53D48 */
/* size: 652 */
int __fastcall sub_53D48(int a1, int a2, int a3)
{
  int v6; // r0
  int v7; // r11
  unsigned __int8 *v9; // r9
  unsigned __int8 *v10; // r6
  unsigned __int8 v11; // r3
  unsigned int v12; // r7
  int v13; // r0
  int v14; // r0
  int v15; // r0
  unsigned int v16; // r5
  unsigned __int8 *v17; // r4
  int v18; // r1
  int v19; // r0
  _BYTE v20[4]; // [sp+0h] [bp-90h] BYREF
  int v21; // [sp+4h] [bp-8Ch] BYREF
  int v22; // [sp+8h] [bp-88h]
  _DWORD v23[2]; // [sp+10h] [bp-80h] BYREF
  char v24[16]; // [sp+18h] [bp-78h] BYREF
  void *v25[2]; // [sp+28h] [bp-68h] BYREF
  _BYTE v26[8]; // [sp+30h] [bp-60h] BYREF
  _DWORD v27[2]; // [sp+38h] [bp-58h] BYREF
  char v28; // [sp+40h] [bp-50h]
  int v29; // [sp+48h] [bp-48h] BYREF
  void *v30; // [sp+4Ch] [bp-44h] BYREF
  unsigned __int8 v31; // [sp+58h] [bp-38h]
  int v32; // [sp+5Ch] [bp-34h]
  int v33; // [sp+60h] [bp-30h] BYREF

  v22 = a2;
  if ( a2 == 0 || a3 == 0 )
    __debugbreak();
  sub_54388(a1 + 16, *(_DWORD *)(a1 + 16), *(_DWORD *)(a1 + 20));
  v6 = (*(int (__fastcall **)(int, int, _DWORD, _DWORD))(*(_DWORD *)a2 + 8))(a2, a3, 0, 0);
  v7 = v6;
  if ( v6 == 0 )
    return 0;
  v9 = (unsigned __int8 *)(*(int (__fastcall **)(int))(*(_DWORD *)v6 + 12))(v6);
  v10 = v9 + 2;
  sub_54494(a1 + 16, *v9);
  v11 = v9[1];
  *(_BYTE *)(a1 + 28) = v11;
  v23[0] = 0;
  v23[1] = 0;
  sub_34844((_DWORD *)(a1 + 4), v11, (int)v23);
  v27[0] = 0;
  v27[1] = 0;
  v28 = 0;
  sub_34890((_DWORD *)(a1 + 32), *(unsigned __int8 *)(a1 + 28), (int)v27);
  if ( *v9 != 0 )
  {
    LOBYTE(v12) = 0;
    do
    {
      sub_4AEA0(&v29);
      LOBYTE(v29) = *v10;
      memcpy(v24, v10 + 1, 0xAu);
      v13 = sub_12F04(v24);
      sub_1318C(&v30, v24, (int)&v24[2 * v13]);
      v31 = v10[11];
      v14 = muls(v10[12] | (v10[13] << 8) | (v10[14] << 16) | (v10[15] << 24), 1199570944);
      v15 = stoi(v14);
      v25[0] = nullptr;
      v32 = v15;
      v25[1] = nullptr;
      v10 += 16;
      sub_54410(v26, v20, 0);
      if ( *(_BYTE *)(a1 + 28) != 0 )
      {
        LOBYTE(v16) = 0;
        do
        {
          v17 = v10;
          v18 = *v10;
          v10 += 4;
          v19 = muls(v18 | (v17[1] << 8) | (v17[2] << 16) | (v17[3] << 24), 1199570944);
          v21 = stoi(v19);
          sub_54054(v25, &v21);
          v16 = (unsigned __int8)(v16 + 1);
        }
        while ( v16 < *(unsigned __int8 *)(a1 + 28) );
      }
      sub_4A57C(&v33, (int *)v25);
      sub_540B4(a1 + 16, &v29);
      sub_39E34(v25);
      sub_39E18((int)&v29);
      v12 = (unsigned __int8)(v12 + 1);
    }
    while ( v12 < *v9 );
  }
  (*(void (__fastcall **)(int, int))(*(_DWORD *)v22 + 12))(v22, v7);
  return 1;
}

