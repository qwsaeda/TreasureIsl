/* ea:   0x0003E194 */
/* name: sub_3E194 */
/* size: 292 */
int __fastcall sub_3E194(_DWORD *a1, int a2, int a3, int a4, int a5, int a6, int a7, int a8)
{
  int v10; // r0
  int v11; // r8
  int v13; // r4
  unsigned int v14; // r7
  __int16 *v15; // r4
  unsigned int i; // r6
  __int16 v17; // t1
  __int16 v18; // [sp+0h] [bp-34h] BYREF
  unsigned int v19; // [sp+4h] [bp-30h]
  char v20; // [sp+8h] [bp-2Ch] BYREF

  v10 = (*(int (__fastcall **)(int, wchar_t *, _DWORD, _DWORD))(*(_DWORD *)a2 + 8))(a2, aFontDat, 0, 0);
  v11 = v10;
  if ( v10 != 0 )
  {
    v13 = (*(int (__fastcall **)(int))(*(_DWORD *)v10 + 12))(v10);
    v14 = (unsigned int)((*(int (__fastcall **)(int))(*(_DWORD *)v11 + 4))(v11) - 2) >> 1;
    v15 = (__int16 *)(v13 + 2);
    if ( v14 != 0 )
    {
      for ( i = 0; i < v14; i = (unsigned __int16)(i + 1) )
      {
        v17 = *v15++;
        v19 = i;
        v18 = v17;
        sub_3E2B8(a1 + 2, &v20, &v18);
      }
    }
    (*(void (__fastcall **)(int, int))(*(_DWORD *)a2 + 12))(a2, v11);
    a1[1] = a2;
    *a1 = (*(int (__fastcall **)(int, wchar_t *, _DWORD, _DWORD))(*(_DWORD *)a2 + 8))(a2, aGoldlandFnt, 0, 0);
    __asm { POP             {R4-R11,PC} }
  }
  return 0;
}

