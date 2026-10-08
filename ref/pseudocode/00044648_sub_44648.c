/* ea:   0x00044648 */
/* name: sub_44648 */
/* size: 204 */
int __fastcall sub_44648(_DWORD *a1, int a2, int a3)
{
  _DWORD *v5; // r6
  int v6; // r1
  int (*v7)(); // r2
  int v8; // r1
  int v9; // r4
  _DWORD v11[4]; // [sp+Ch] [bp-34h] BYREF
  _DWORD v12[9]; // [sp+1Ch] [bp-24h] BYREF

  v5 = (_DWORD *)a1[251];
  if ( *v5 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(*v5 + 12);
  if ( v6 == 16 )
  {
    v7 = sub_5BC14;
  }
  else if ( v6 == 8 )
  {
    v7 = sub_5D9C4;
  }
  else
  {
    v7 = nullptr;
  }
  v12[2] = a3;
  v11[1] = 16;
  v8 = a3 + 16;
  v11[3] = 16;
  v12[1] = a2 + 16;
  v9 = a1[1];
  v12[0] = a2;
  v12[3] = v8;
  a1[122] = v7;
  v11[0] = 0;
  v11[2] = 0;
  return (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v9 + 20))(
           v9,
           v12,
           *v5,
           v11,
           v7,
           &dword_81230,
           1);
}

