/* ea:   0x00030BC4 */
/* name: sub_30BC4 */
/* size: 20 */
void __fastcall sub_30BC4(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

