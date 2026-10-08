/* ea:   0x0003E720 */
/* name: sub_3E720 */
/* size: 56 */
void __fastcall sub_3E720(_DWORD *a1, int a2)
{
  void *v4; // r4

  v4 = (void *)*a1;
  if ( a2 != *a1 && v4 != nullptr )
  {
    sub_590A4(*a1);
    operator delete(v4);
  }
  *a1 = a2;
}

