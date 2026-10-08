/* ea:   0x00044828 */
/* name: sub_44828 */
/* size: 336 */
int __fastcall sub_44828(_DWORD *a1, int a2, int a3, int a4)
{
  int v4; // r4
  int result; // r0
  int v7; // r2
  int v8; // r11
  _DWORD *v9; // r7
  int v10; // r1
  int (*v11)(); // r5
  int v12; // r4
  int v13; // r2
  int v14; // [sp+Ch] [bp-64h]
  int v17; // [sp+18h] [bp-58h]
  char v18[16]; // [sp+1Ch] [bp-54h] BYREF
  _DWORD v19[4]; // [sp+2Ch] [bp-44h] BYREF
  _DWORD v20[13]; // [sp+3Ch] [bp-34h] BYREF

  v4 = a4;
  itoa(a2, v18, 10);
  result = strlen(v18);
  v17 = result;
  v7 = 0;
  v14 = 0;
  if ( result > 0 )
  {
    v8 = v4 + 9;
    while ( 1 )
    {
      v9 = (_DWORD *)a1[250];
      if ( *v9 == 0 )
        __debugbreak();
      v10 = *(_DWORD *)(*v9 + 12);
      if ( v10 == 16 )
        v11 = sub_5BC14;
      else
        v11 = v10 == 8 ? sub_5D9C4 : nullptr;
      v20[2] = v4;
      v12 = a1[1];
      v13 = -v18[v7];
      a1[122] = v11;
      v19[0] = 0;
      v20[3] = v8;
      v19[2] = 9 * (v13 + 57);
      v19[3] = 9 * (v13 + 58);
      v19[1] = 8;
      v20[1] = a3 + 8;
      v20[0] = a3;
      (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v12 + 20))(
        v12,
        v20,
        *v9,
        v19,
        v11,
        &dword_81230,
        1);
      result = v14 + 1;
      v14 = result;
      a3 += 8;
      if ( result >= v17 )
        break;
      v7 = result;
      v4 = a4;
    }
  }
  return result;
}

