/* ea:   0x0004F3F4 */
/* name: sub_4F3F4 */
/* size: 400 */
int __fastcall sub_4F3F4(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int result; // r0
  int v8; // r11
  int v9; // r6
  signed int v10; // r4
  int (*v11)(); // r10
  int v12; // r5
  int v13; // r0
  int v14; // r0
  int v15; // r2
  int v16; // r1
  int v17; // r1
  int v18; // r0
  int v19; // r1
  int (**v20)(); // r0
  _DWORD v22[4]; // [sp+14h] [bp-44h] BYREF
  _DWORD v23[13]; // [sp+24h] [bp-34h] BYREF

  result = sub_11118(a1, a4, a5);
  if ( (_BYTE)result != 0 )
  {
    v8 = a1[1];
    if ( v8 == 0 )
      __debugbreak();
    v9 = *(_DWORD *)(a1[1] + 4);
    if ( v8 == 0 )
      __debugbreak();
    v10 = a1[4];
    v11 = nullptr;
    v12 = *(_DWORD *)(v8 + 8) / (unsigned int)v10;
    v13 = a1[6];
    if ( v13 <= 5 )
    {
      v16 = a1[5];
      a1[6] = v13 + 1;
      if ( v16 >= v10 )
      {
        a1[5] = 0;
        v15 = 0;
      }
      else
      {
        v15 = v16 * v12;
      }
    }
    else
    {
      v14 = a1[5];
      a1[6] = 0;
      if ( v14 >= v10 )
      {
        a1[5] = 0;
        v15 = 0;
        a1[5] = 1;
      }
      else
      {
        v15 = v14 * v12;
        a1[5] = v14 + 1;
      }
    }
    v17 = a1[3];
    v22[3] = v15 + v12;
    v18 = a1[2];
    v22[2] = v15;
    v22[0] = 0;
    v22[1] = v9;
    v23[0] = v18 + a4;
    v23[1] = v18 + v9 + a4;
    v23[2] = v17 + a5;
    v23[3] = v17 + v12 + a5;
    if ( v8 == 0 )
      __debugbreak();
    v19 = *(_DWORD *)(v8 + 12);
    if ( v19 == 16 )
    {
      v20 = off_7BB78;
    }
    else
    {
      if ( v19 != 8 )
        return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
                 a2,
                 v23,
                 v8,
                 v22,
                 v11,
                 &dword_81230,
                 1);
      v20 = off_7BBB8;
    }
    v11 = v20[8];
    return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
             a2,
             v23,
             v8,
             v22,
             v11,
             &dword_81230,
             1);
  }
  return result;
}

