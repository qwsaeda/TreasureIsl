/* ea:   0x000386B0 */
/* name: sub_386B0 */
/* size: 20 */
void __fastcall sub_386B0(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

