/* ea:   0x00044714 */
/* name: sub_44714 */
/* size: 276 */
int __fastcall sub_44714(_DWORD *a1, int a2, int a3, int a4)
{
  _DWORD *v7; // r7
  int v8; // r3
  int v9; // r1
  int (*v10)(); // r6
  int v11; // r5
  int v12; // r4
  int result; // r0
  _DWORD v15[4]; // [sp+14h] [bp-44h] BYREF
  _DWORD v16[13]; // [sp+24h] [bp-34h] BYREF

  do
  {
    v7 = (_DWORD *)a1[250];
    v8 = a2 % 10;
    a2 /= 10;
    if ( *v7 == 0 )
      __debugbreak();
    v9 = *(_DWORD *)(*v7 + 12);
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
    v15[0] = 0;
    v16[1] = a3;
    a1[122] = v10;
    v15[2] = 9 * (9 - v8);
    v15[3] = 9 * (10 - v8);
    v15[1] = 8;
    v11 = a3 - 8;
    v12 = a1[1];
    v16[2] = a4;
    v16[3] = a4 + 9;
    v16[0] = v11;
    result = (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v12 + 20))(
               v12,
               v16,
               *v7,
               v15,
               v10,
               &dword_81230,
               1);
    a3 = v11;
  }
  while ( a2 != 0 );
  return result;
}

