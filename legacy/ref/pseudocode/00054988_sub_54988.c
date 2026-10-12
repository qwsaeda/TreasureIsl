/* ea:   0x00054988 */
/* name: sub_54988 */
/* size: 212 */
int __fastcall sub_54988(_DWORD *a1, int a2, int a3, int a4)
{
  _DWORD *v6; // r7
  int v7; // r1
  int (*v8)(); // r2
  int v9; // r4
  _DWORD v11[4]; // [sp+Ch] [bp-3Ch] BYREF
  _DWORD v12[11]; // [sp+1Ch] [bp-2Ch] BYREF

  v6 = (_DWORD *)a1[264];
  if ( *v6 == 0 )
    __debugbreak();
  v7 = *(_DWORD *)(*v6 + 12);
  if ( v7 == 16 )
  {
    v8 = sub_5BC14;
  }
  else if ( v7 == 8 )
  {
    v8 = sub_5D9C4;
  }
  else
  {
    v8 = nullptr;
  }
  v12[0] = a2;
  v11[3] = 16 * a4;
  v11[2] = 16 * a4 - 16;
  v12[1] = a2 + 16;
  v9 = a1[1];
  v12[3] = a3 + 16;
  a1[122] = v8;
  v11[0] = 0;
  v11[1] = 16;
  v12[2] = a3;
  return (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v9 + 20))(
           v9,
           v12,
           *v6,
           v11,
           v8,
           &dword_81230,
           1);
}

