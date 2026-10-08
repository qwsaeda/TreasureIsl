/* ea:   0x0004E1B4 */
/* name: sub_4E1B4 */
/* size: 352 */
int __fastcall sub_4E1B4(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int result; // r0
  int v8; // r7
  int v9; // r6
  int v10; // r0
  int v11; // r1
  int v12; // r2
  _DWORD v14[13]; // [sp+24h] [bp-34h] BYREF

  result = sub_11118(a1, a4, a5);
  if ( (_BYTE)result != 0 )
  {
    result = sub_4E314(a1);
    if ( (_BYTE)result != 0 )
    {
      if ( a1[1] == 0 )
        __debugbreak();
      v8 = *(_DWORD *)(a1[1] + 4);
      if ( a1[1] == 0 )
        __debugbreak();
      v9 = *(_DWORD *)(a1[1] + 8) / a1[4];
      if ( a1[5] >= a1[4] )
        a1[5] = 0;
      ++a1[5];
      v10 = a1[2];
      v11 = a1[3];
      v14[0] = v10 + a4;
      v14[1] = v10 + v8 + a4;
      v12 = a1[1];
      v14[2] = v11 + a5;
      v14[3] = v11 + v9 + a5;
      if ( v12 == 0 )
        __debugbreak();
      return (*(int (__fastcall **)(int, _DWORD *))(*(_DWORD *)a2 + 20))(a2, v14);
    }
  }
  return result;
}

