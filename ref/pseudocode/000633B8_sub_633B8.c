/* ea:   0x000633B8 */
/* name: sub_633B8 */
/* size: 324 */
int __fastcall sub_633B8(int a1, int a2)
{
  int *v4; // r0
  int v5; // r2
  int v6; // r1
  int v7; // r6
  int v8; // r5
  int v9; // r4
  _DWORD *v10; // r0
  int v12; // r3
  char v13[16]; // [sp+0h] [bp-2Ch] BYREF
  int v14; // [sp+10h] [bp-1Ch]
  int v15; // [sp+14h] [bp-18h]

  if ( a2 == 0 )
    __debugbreak();
  if ( *(_BYTE *)(a1 + 4) != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
  if ( GXOpenDisplay(a2, 1) != 0 )
  {
    v4 = (int *)GXGetDisplayProperties(v13);
    v5 = v4[2];
    v6 = v4[5];
    v7 = *v4;
    v8 = v4[1];
    v9 = v4[3];
    v14 = v4[4];
    v15 = v6;
    *(_DWORD *)(a1 + 36) = v5;
    *(_DWORD *)(a1 + 40) = v9;
    v10 = sub_62694(v7, v8, 16, 1);
    *(_DWORD *)(a1 + 8) = v10;
    if ( v10 != nullptr )
    {
      sub_63544(a1);
      (*(void (__fastcall **)(int, int))(*(_DWORD *)a1 + 28))(a1, a1 + 12);
      v12 = *(_DWORD *)(a1 + 32);
      if ( v12 != 0 )
      {
        if ( v12 == 1 )
        {
          *(_BYTE *)(a1 + 4) = 1;
          *(_DWORD *)(a1 + 48) = sub_635E8;
          return 1;
        }
      }
      else
      {
        *(_BYTE *)(a1 + 4) = 1;
        *(_DWORD *)(a1 + 48) = sub_635D0;
      }
      return *(unsigned __int8 *)(a1 + 4);
    }
  }
  return 0;
}

