/* ea:   0x00065990 */
/* name: sub_65990 */
/* size: 228 */
void __fastcall sub_65990(_DWORD *a1, void *a2)
{
  int v4; // r0
  void *v5; // [sp+0h] [bp-14h] BYREF
  void *v6; // [sp+4h] [bp-10h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  (*(void (__fastcall **)(void *))(*(_DWORD *)a2 + 24))(a2);
  v4 = (*(int (__fastcall **)(void *))(*(_DWORD *)a2 + 20))(a2);
  if ( v4 == 1 )
  {
    if ( a1[4] > (unsigned int)((a1[9] - a1[8]) >> 2) )
    {
      v6 = a2;
      sub_65B14(a1 + 8, &v6);
      return;
    }
  }
  else
  {
    if ( v4 != 2 )
    {
      __debugbreak();
      return;
    }
    if ( a1[4] > (unsigned int)((a1[6] - a1[5]) >> 2) )
    {
      v5 = a2;
      sub_65AB4(a1 + 5, &v5);
      return;
    }
  }
  *(_DWORD *)a2 = off_7BD34;
  operator delete(a2);
}

