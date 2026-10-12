/* ea:   0x0001EA70 */
/* name: sub_1EA70 */
/* size: 272 */
int __fastcall sub_1EA70(_DWORD *a1, int a2, int a3, int a4)
{
  int result; // r0
  int v9; // r1
  int (*v10)(); // r1
  int v11; // r5
  int v12; // r4
  int v13; // r1
  int v14; // r0
  _DWORD v15[4]; // [sp+Ch] [bp-44h] BYREF
  _DWORD v16[13]; // [sp+1Ch] [bp-34h] BYREF

  result = a1[4];
  if ( result != 0 )
  {
    v9 = *(_DWORD *)(result + 12);
    if ( v9 == 16 )
    {
      v10 = sub_5BC14;
    }
    else if ( v9 == 8 )
    {
      v10 = sub_5D9C4;
    }
    else
    {
      v10 = nullptr;
    }
    a1[3] = v10;
    while ( 1 )
    {
      v16[1] = a3;
      v11 = a3 - 8;
      v16[2] = a4;
      v12 = a1[2];
      v16[0] = v11;
      v15[1] = 8;
      v16[3] = a4 + 9;
      v13 = 10 * (a2 / 10) - a2;
      a2 /= 10;
      v15[2] = 9 * (v13 + 9);
      v15[3] = 9 * (v13 + 10);
      v14 = a1[3];
      v15[0] = 0;
      result = (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int, int *, int))(*(_DWORD *)v12 + 20))(
                 v12,
                 v16,
                 a1[4],
                 v15,
                 v14,
                 &dword_81230,
                 1);
      if ( a2 == 0 )
        break;
      a3 = v11;
    }
  }
  return result;
}

